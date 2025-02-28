#[cfg(feature = "derive")]
mod from_row;

#[cfg(feature = "derive")]
#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_row::expand_derive_from_row(input.into())
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[cfg(feature = "query")]
mod query;

#[cfg(feature = "query")]
#[proc_macro]
pub fn expand_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::expand_input(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
