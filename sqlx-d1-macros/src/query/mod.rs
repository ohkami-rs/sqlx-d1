mod input;
mod output;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use syn::LitStr;

struct Location {
    manifest_dir: PathBuf,
    workspace_root: LazyLock<PathBuf>,
}
/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/query/mod.rs#L80-L114>
static LOCATION: LazyLock<Location> = LazyLock::new(|| {
    fn get_manifest_dir() -> PathBuf {
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("`CARGO_MANIFEST_DIR` must be set")
            .into()
    }

    fn get_workspace_root() -> PathBuf {
        use serde::Deserialize;
        use std::process::Command;

        let cargo = std::env::var("CARGO").expect("`CARGO` must be set");

        let output = Command::new(&cargo)
            .args(&["metadata", "--format-version=1", "--no-deps"])
            .current_dir(&get_manifest_dir())
            .env_remove("__CARGO_FIX_PLZ")
            .output()
            .expect("Could not fetch metadata");

        #[derive(Deserialize)]
        struct CargoMetadata {
            workspace_root: PathBuf,
        }

        let cargo_metadata: CargoMetadata =
            serde_json::from_slice(&output.stdout).expect("Invalid `cargo metadata` output");

        cargo_metadata.workspace_root
    }

    Location {
        manifest_dir: get_manifest_dir(),
        workspace_root: LazyLock::new(get_workspace_root),
    }
});
impl Location {
    fn miniflare_sqlite_file(&self) -> Result<Option<PathBuf>, io::Error> {
        fn miniflare_d1_dir_path_in_parent(parent_path: impl AsRef<Path>) -> PathBuf {
            parent_path
                .as_ref()
                .join(".wrangler")
                .join("state")
                .join("v3")
                .join("d1")
                .join("miniflare-D1DatabaseObject")
        }

        let miniflare_d1_dir = 'search: {
            for parent_candidate in [&*LOCATION.manifest_dir, &*LOCATION.workspace_root] {
                let candidate = miniflare_d1_dir_path_in_parent(parent_candidate);
                if std::fs::exists(&candidate)? && candidate.is_dir() {
                    break 'search candidate;
                }
            }
            return Ok(None);
        };

        let mut sqlite_files = std::fs::read_dir(miniflare_d1_dir)?
            .filter_map(|r| r.as_ref().ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sqlite"))
            .collect::<Vec<_>>();

        match sqlite_files.len() {
            0 => Ok(None),
            1 => Ok(sqlite_files.pop()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Multiple miniflare's D1 emulators are found! \
                Sorry, sqlx_d1 only supports single D1 binding now.",
            )),
        }
    }

    fn dot_sqlx_dir(&self) -> Result<Option<DotSqlx>, io::Error> {
        for parent_candidate in [&*LOCATION.manifest_dir, &*LOCATION.workspace_root] {
            if let Some(it) = DotSqlx::find_in_parent(parent_candidate)? {
                return Ok(Some(it));
            }
        }
        Ok(None)
    }
}

struct DotSqlx(PathBuf);
impl DotSqlx {
    fn find_in_parent(parent_dir: &Path) -> Result<Option<Self>, io::Error> {
        let candidate = parent_dir.join(".sqlx");
        if std::fs::exists(&candidate)? && candidate.is_dir() {
            Ok(Some(DotSqlx(candidate)))
        } else {
            Ok(None)
        }
    }

    fn file_path_of(&self, sql: &str) -> PathBuf {
        /* ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/query/data.rs#L193-L198> */
        let hash = {
            use sha2::{Digest, Sha256};
            ::hex::encode(Sha256::digest(sql.as_bytes()))
        };

        /* ref:
            <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/query/mod.rs#L165>
            <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/query/data.rs#L156>
        */
        let file_name = format!("query-{hash}.json");

        self.0.join(file_name)
    }

    fn get_cached_describe_of(
        &self,
        sql: &str,
    ) -> Result<Option<sqlx_core::describe::Describe<sqlx_d1_core::D1>>, io::Error> {
        match std::fs::read(self.file_path_of(sql)) {
            Ok(bytes) => {
                let describe = ::serde_json::from_slice(&bytes).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("failed to parse the query cache of `{sql}`: {e}"),
                    )
                })?;
                Ok(Some(describe))
            }
            Err(e) => {
                if matches!(e.kind(), io::ErrorKind::NotFound) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/query/data.rs#L153-L190>
    fn cache_describe(
        &self,
        sql: &str,
        describe: sqlx_core::describe::Describe<sqlx_d1_core::D1>,
    ) -> Result<(), io::Error> {
        let describe = ::serde_json::to_vec(&describe).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to serialize the query cache of `{sql}`: {e}"),
            )
        })?;
        std::fs::write(self.file_path_of(sql), describe)?;
        Ok(())
    }
}

pub(super) fn expand_input(input: TokenStream) -> Result<TokenStream, syn::Error> {
    use sqlx_core::executor::Executor;
    use sqlx_d1_core::D1Connection;
    // Assumeing the cost of context switching is almost the same
    // or larger than that of synchronous blocking in this case
    use std::sync::Mutex;

    let input = syn::parse2::<self::input::QueryMacroInput>(input)?;

    let describe = match LOCATION.miniflare_sqlite_file().map_err(|e| syn::Error::new(Span::call_site(), e))? {
        Some(sqlite_file_path) => {
            static CONNECTION: OnceLock<Result<Mutex<D1Connection>, syn::Error>> = OnceLock::new();

            let mut conn = CONNECTION.get_or_init(|| {
                futures_lite::future::block_on(async {
                    let conn = D1Connection::connect(&format!("sqlite://{}", sqlite_file_path.display()))
                        .await
                        .map_err(|e| syn::Error::new(Span::call_site(), e))?;
                    Ok(Mutex::new(conn))
                })
            }).as_ref().map_err(Clone::clone)?.lock().map_err(|_| syn::Error::new(
                input.src_span,
                "Bug or invalid use of `sqlx_d1`. Be sure to use `sqlx_d1` where the target is set to \
                `wasm32-unknown-unknown` ! \n\
                For this, typcally, place `.cargo/config.toml` of following content at the root of \
                your project or workspace : \n\
                \n\
                [build]\n\
                target = \"wasm32-unknown-unknown\"\n
                \n\
                If you think this of a bug, please let me know the situation in \
                GitHub Issues (https://ohkami-rs/sqlx-d1/issues) !"
            ))?;

            futures_lite::future::block_on(async {
                let describe = (&mut *conn).describe(&input.sql).await;
                drop(conn);
                describe
            }).map_err(|e| syn::Error::new(input.src_span, e))?
        }

        None => match LOCATION.dot_sqlx_dir().map_err(|e| syn::Error::new(input.src_span, e))? {
            Some(dot_sqlx_dir) => dot_sqlx_dir
                .get_cached_describe_of(&input.sql)
                .map_err(|e| syn::Error::new(input.src_span, e))?
                .ok_or_else(|| syn::Error::new(
                    input.src_span,
                    "there is no cached data for this query, run `cargo sqlx prepare` to update the query cache"
                ))?,

            None => return Err(syn::Error::new(
                input.src_span,
                "Neither miniflare D1 emulator nor .sqlx directory is found ! \n\
                For setting up miniflare, run \
                `wrangler d1 migrations create <BINDING> <MIGRATION>` and \
                `wrangler d1 migrations apply <BINDING> --local`.\n\
                For setting up .sqlx directory for offline mode, \
                run `cargo sqlx prepare` where `cargo sqlx` is installed and \
                miniflare D1 emulator is accessable (offen your local PC)."
            ))
        }
    };

    compare_expand(input, describe)
}

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/query/mod.rs#L241-379>
fn compare_expand(
    input: self::input::QueryMacroInput,
    describe: sqlx_core::describe::Describe<sqlx_d1_core::D1>,
) -> Result<TokenStream, syn::Error> {
    if let Some(actual_params) = describe.parameters() {
        use sqlx_core::Either;

        let n_input_params = input.arg_exprs.len();
        let n_correct_params = match actual_params {
            Either::Left(params) => params.len(),
            Either::Right(n) => n,
        };

        let _: () = (n_input_params == n_correct_params)
            .then_some(())
            .ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    format!("expected {n_correct_params} parameters, got {n_input_params}"),
                )
            })?;
    }

    let args_tokens = input.quote_args_with(&describe)?;

    let query_args_ident = format_ident!("query_args");

    let output = if describe.columns().iter().all({
        use sqlx_core::{column::Column as _, type_info::TypeInfo as _};
        |c| c.type_info().is_void()
    }) {
        let sql = LitStr::new(&input.sql, input.src_span);
        quote! {
            ::sqlx_d1::query_with(#sql, #query_args_ident)
        }
    } else {
        match &input.record_type {
            input::RecordType::Scalar => {
                output::quote_query_scalar(&input, &query_args_ident, &describe)?
            }
            input::RecordType::Given(out_ty) => {
                let columns = output::columns_to_rust(&describe)?;
                output::quote_query_as(&input, out_ty, &query_args_ident, &columns)
            }
            input::RecordType::Generated => {
                let columns = self::output::columns_to_rust(&describe)?;

                let record_type_name_token = syn::parse_str::<syn::Type>("Record").unwrap();

                for rust_column in &columns {
                    if rust_column.type_.is_wildcard() {
                        return Err(syn::Error::new(
                            rust_column.ident.span(),
                            "wildcard overrides are only allowed with an explicit record type, \
                            e.g. `query_as!()` and its variants",
                        ));
                    }
                }

                let record_fields = columns.iter().map(|rc| {
                    let (ident, type_) = (&rc.ident, &rc.type_);
                    quote! {
                        #ident: #type_,
                    }
                });

                let mut record_tokens = quote! {
                    #[derive(Debug)]
                    struct #record_type_name_token {
                        #(#record_fields)*
                    }
                };
                record_tokens.extend(output::quote_query_as(
                    &input,
                    &record_type_name_token,
                    &query_args_ident,
                    &columns,
                ));

                record_tokens
            }
        }
    };

    if let Some(dot_sqlx_dir) = LOCATION
        .dot_sqlx_dir()
        .map_err(|e| syn::Error::new(input.src_span, e))?
    {
        dot_sqlx_dir
            .cache_describe(&input.sql, describe)
            .map_err(|e| syn::Error::new(input.src_span, e))?;
    }

    Ok(quote! {
        {
            #[allow(clippy::all)]
            {
                use ::sqlx_d1::sqlx_core::arguments::Arguments as _;

                #args_tokens

                #output
            }
        }
    })
}
