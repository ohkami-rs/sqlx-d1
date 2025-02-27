mod input;
mod output;

use std::io;
use std::sync::{LazyLock, OnceLock, Once};
use std::path::{Path, PathBuf};
use proc_macro2::{TokenStream, Span};
use syn::LitStr;
use quote::{quote, format_ident};

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
        workspace_root: LazyLock::new(get_workspace_root)
    }
});
impl Location {
    fn miniflare_sqlite_file(&self) -> Result<PathBuf, io::Error> {
        fn miniflare_d1_dir_in_a_package_root(package_root: impl AsRef<Path>) -> PathBuf {
            package_root.as_ref()
                .join(".wrangler")
                .join("state")
                .join("v3")
                .join("d1")
                .join("miniflare-D1DatabaseObject")
        }
        
        let miniflare_d1_dir = {
            ({
                let candidate = miniflare_d1_dir_in_a_package_root(&*LOCATION.manifest_dir);
                std::fs::exists(&candidate)?.then_some(candidate)
            })
            .or_else(|| {
                let candidate = miniflare_d1_dir_in_a_package_root(&*LOCATION.workspace_root);
                std::fs::exists(&candidate).ok()?.then_some(candidate)
            })
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::NotFound,
                "Miniflare's D1 emulator is not found. Make sure to run \
                `wrangler d1 migrations create <BINDING> <MIGRATION>` and \
                `wrangler d1 migrations apply <BINDING> --local`."
            ))?
        };
    
        let mut sqlite_files = std::fs::read_dir(miniflare_d1_dir)?
            .filter_map(|r| r.as_ref().ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sqlite"))
            .collect::<Vec<_>>();
    
        match sqlite_files.len() {
            0 => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No Miniflare's D1 emulator is found! Make sure to run \
                `wrangler d1 migrations create <BINDING> <MIGRATION>` and \
                `wrangler d1 migrations apply <BINDING> --local`."
            )),
            2.. => Err(io::Error::new(
                io::ErrorKind::Other,
                "Multiple Miniflare's D1 emulators are found! \
                Sorry, sqlx_d1 only supports single D1 binding now."
            )),
            1 => Ok(sqlite_files.pop().unwrap())
        }
    }
}

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros/src/lib.rs#L9-L23>
#[proc_macro]
pub fn expand_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    self::expand_input(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_input(input: TokenStream) -> Result<TokenStream, syn::Error> {
    use sqlx_d1_core::D1Connection;
    use sqlx_core::{executor::Executor, describe::Describe};
    use std::sync::Arc;
    // Assumeing the cost of context switching is almost the same
    // or larger than that of synchronous blocking in this case
    use std::sync::Mutex;
    
    let input = syn::parse2::<self::input::QueryMacroInput>(input)?;
    
    let describe = {
        static CONNECTION: OnceLock<Result<Mutex<D1Connection>, syn::Error>> = OnceLock::new();

        let mut conn = CONNECTION.get_or_init(|| {
            let sqlite_file_path = LOCATION
                .miniflare_sqlite_file()
                .map_err(|e| syn::Error::new(Span::call_site(), e))?;
                
            futures_lite::future::block_on(async {
                let conn = D1Connection::connect(&format!("sqlite://{}", sqlite_file_path.display()))
                    .await
                    .map_err(|e| syn::Error::new(Span::call_site(), e))?;
                Ok(Mutex::new(conn))
            })
        }).as_ref().map_err(Clone::clone)?.lock().unwrap();

        futures_lite::future::block_on(async {
            conn.describe(&input.sql).await
        }).map_err(|e| syn::Error::new(input.src_span, e))?
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
            .ok_or_else(|| syn::Error::new(Span::call_site(), format!(
                "expected {n_correct_params} parameters, got {n_input_params}"
            )))?;
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
                output::quote_query_scalar(
                    &input,
                    &query_args_ident,
                    &describe,
                )?
            }
            input::RecordType::Given(out_ty) => {
                let columns = output::columns_to_rust(&describe)?;
                output::quote_query_as(
                    &input,
                    out_ty,
                    &query_args_ident,
                    &columns
                )
            }
            input::RecordType::Generated => {
                let columns = self::output::columns_to_rust(&describe)?;

                let record_type_name_token = syn::parse_str::<syn::Type>("Record").unwrap();

                for rust_column in &columns {
                    if rust_column.type_.is_wildcard() {
                        return Err(syn::Error::new(
                            rust_column.ident.span(),
                            "wildcard overrides are only allowed with an explicit record type, \
                            e.g. `query_as!()` and its variants"
                        ))
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

    Ok(quote! {
        {
            #[allow(clippy::all)]
            {
                use ::sqlx_d1::sqlx_core::Arguments as _;

                #args_tokens

                #output
            }
        }
    })
}

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/database/mod.rs>

use sqlx_core::{types::Type, database::Database};
use sqlx_d1_core::D1;

macro_rules! input_ty {
    ($ty:ty, $input:ty) => {
        stringify!($input)
    };
    ($ty:ty) => {
        stringify!($ty)
    };
}

macro_rules! type_names {
    ( $( $(#[$meta:meta])? $T:ty $(| $input:ty)? ),* $(,)? ) => {
        fn param_type_name_for_info(
            info: &<D1 as Database>::TypeInfo,
        ) -> Option<&'static str> {
            $(
                $(#[$meta])?
                if
                    <$T as Type<D1>>::type_info() == *info ||
                    <$T as Type<D1>>::compatible(info)
                {
                    return Some(input_ty!($T $(, $input)?));
                }
            )*
            None
        }

        fn return_type_name_for_info(
            info: &<D1 as Database>::TypeInfo,
        ) -> Option<&'static str> {
            $(
                $(#[$meta])?
                if
                    <$T as Type<D1>>::type_info() == *info ||
                    <$T as Type<D1>>::compatible(info)
                {
                    return Some(stringify!($T));
                }
            )*
            None
        }
    };
}

type_names! {
    bool,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    String | &str,
    Vec<u8> | &[u8],

    #[cfg(feature = "uuid")]
    sqlx_core::types::Uuid,
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
