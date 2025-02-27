use sqlx_d1_core::D1;
use sqlx_macros_core::query::{expand_input, QueryMacroInput, QueryDriver};
use quote::quote;

#[proc_macro]
pub fn expand_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {    
    /* ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros/src/lib.rs#L9-L23> */

    match expand_input(
        syn::parse_macro_input!(input as QueryMacroInput),
        &[QueryDriver::new::<D1>()]
    ) {
        Ok(ts) => ts.into(),
        Err(e) => e.downcast_ref::<syn::Error>()
            .map(|parse_error| {
                parse_error.to_compile_error().into()
            })
            .unwrap_or_else({
                let msg = e.to_string();
                quote!(::std::compile_error!(#msg)).into()
            })
    }
}
