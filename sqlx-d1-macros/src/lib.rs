use sqlx_macros_core::query::{QueryMacroInput, QueryDriver};
use sqlx_sqlite::Sqlite;
use std::sync::{LazyLock, Once};
use std::path::{Path, PathBuf};
use proc_macro2::{TokenStream, Span};
use syn::spanned::Spanned;

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros/src/lib.rs#L9-L23>
#[proc_macro]
pub fn expand_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    self::expand_input(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/query/mod.rs#L80-L106>
fn workspace_root_of_a_package_root(manifest_dir: impl AsRef<Path>) -> PathBuf {
    use serde::Deserialize;
    use std::process::Command;

    let cargo = std::env::var("CARGO").expect("`CARGO` must be set");

    let output = Command::new(&cargo)
        .args(&["metadata", "--format-version=1", "--no-deps"])
        .current_dir(manifest_dir.as_ref())
        .env_remove("__CARGO_FIX_PLZ")
        .output()
        .expect("Could not fetch metadata");

    #[derive(Deserialize)]
    struct CargoMetadata {
        workspace_root: PathBuf,
    }

    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).expect("Invalid `cargo metadata` output");

    metadata.workspace_root
}

fn miniflare_d1_dir_in_a_package_root(package_root: impl AsRef<Path>) -> PathBuf {
    package_root.as_ref()
        .join(".wrangler")
        .join("state")
        .join("v3")
        .join("d1")
        .join("miniflare-D1DatabaseObject")
}

fn expand_input(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let sqlite_file_path = {
        let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
            .expect("`CARGO_MANIFEST_DIR` must be set")
            .into();
        
        let workspace_root = LazyLock::new(|| {
            workspace_root_of_a_package_root(&manifest_dir)
        });
    
        let miniflare_d1_dir = {
            ({
                let candidate = miniflare_d1_dir_in_a_package_root(&*manifest_dir);
                std::fs::exists(&candidate).is_ok_and(|e|e).then_some(candidate)
            })
            .or_else(|| {
                let candidate = miniflare_d1_dir_in_a_package_root(&*workspace_root);
                std::fs::exists(&candidate).is_ok_and(|e|e).then_some(candidate)
            })
            .ok_or_else(|| syn::Error::new(Span::call_site(),
                "Miniflare's D1 emulator is not found. Make sure to run \
                `wrangler d1 migrations create <BINDING> <MIGRATION>` and \
                `wrangler d1 migrations apply <BINDING> --local`."
            ))?
        };
    
        let mut sqlite_files = std::fs::read_dir(miniflare_d1_dir)
            .map_err(|e| syn::Error::new(Span::call_site(), format!(
                "Failed to read Miniflare's D1 emulator: {e}"
            )))?
            .filter_map(|r| r.as_ref().ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "sqlite"))
            .collect::<Vec<_>>();
    
        match sqlite_files.len() {
            0 => return Err(syn::Error::new(Span::call_site(),
                "No Miniflare's D1 emulator is found! Make sure to run \
                `wrangler d1 migrations create <BINDING> <MIGRATION>` and \
                `wrangler d1 migrations apply <BINDING> --local`."
            )),
            2.. => return Err(syn::Error::new(Span::call_site(),
                "Multiple Miniflare's D1 emulators are found! \
                Sorry, sqlx_d1 only supports single D1 binding now."
            )),
            1 => sqlite_files.pop().unwrap()
        }
    };

    static SET_DATABASE_URL: Once = Once::new();
    SET_DATABASE_URL.call_once(|| unsafe {
        /* SAFETY: call in `Once::call_once` */
        std::env::set_var("DATABASE_URL", format!("sqlite://{}", sqlite_file_path.display()));
    });

    /*
        now environment variable `DATABASE_URL` points to the sqlite file for
        Miniflare's D1 emulator, let's call original `expand_input` for SQLite...
    */

    let span = input.span();

    let qinput = syn::parse2::<QueryMacroInput>(input)?;
    let driver = QueryDriver::new::<Sqlite>();

    sqlx_macros_core::query::expand_input(qinput, &[driver])
        .map_err(|e| syn::Error::new(span, e.to_string()))
}
