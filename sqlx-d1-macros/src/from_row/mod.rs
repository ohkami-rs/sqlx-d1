//! ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/derives/row.rs>

mod attributes;

use self::attributes::{parse_child_attributes, parse_container_attributes};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Expr, Field,
    Fields, FieldsNamed, FieldsUnnamed, Lifetime, Stmt,
};

pub fn expand_derive_from_row(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;

    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_from_row_struct(&input, named),

        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => expand_derive_from_row_struct_unnamed(&input, unnamed),

        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Err(syn::Error::new_spanned(
            input,
            "unit structs are not supported",
        )),

        Data::Enum(_) => Err(syn::Error::new_spanned(input, "enums are not supported")),

        Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    }
}

fn expand_derive_from_row_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(R: ::sqlx_d1::sqlx_core::row::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(&#lifetime ::std::primitive::str: ::sqlx_d1::sqlx_core::column::ColumnIndex<R>));

    let container_attributes = parse_container_attributes(&input.attrs)?;

    let default_instance: Option<Stmt> = if container_attributes.default {
        predicates.push(parse_quote!(#ident: ::std::default::Default));
        Some(parse_quote!(
            let __default = #ident::default();
        ))
    } else {
        None
    };

    let reads: Vec<Stmt> = fields
        .iter()
        .filter_map(|field| -> Option<Stmt> {
            let id = &field.ident.as_ref()?;
            let attributes = parse_child_attributes(&field.attrs).unwrap();
            let ty = &field.ty;

            if attributes.skip {
                return Some(parse_quote!(
                    let #id: #ty = Default::default();
                ));
            }

            let id_s = if let Some(s) = attributes.rename {
                s
            } else {
                let s = id.to_string().trim_start_matches("r#").to_owned();
                match container_attributes.rename_all {
                    Some(rename_all) => rename_all.apply(&s),
                    None => s
                }
            };

            let expr: Expr = match (attributes.flatten, attributes.try_from, attributes.json) {
                // <No attributes>
                (false, None, false) => {
                    predicates
                        .push(parse_quote!(#ty: ::sqlx_d1::sqlx_core::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(#ty: ::sqlx_d1::sqlx_core::types::Type<R::Database>));

                    parse_quote!(__row.try_get(#id_s))
                }
                // Flatten
                (true, None, false) => {
                    predicates.push(parse_quote!(#ty: ::sqlx_d1::FromRow<#lifetime, R>));
                    parse_quote!(<#ty as ::sqlx_d1::FromRow<#lifetime, R>>::from_row(__row))
                }
                // Flatten + Try from
                (true, Some(try_from), false) => {
                    predicates.push(parse_quote!(#try_from: ::sqlx_d1::FromRow<#lifetime, R>));
                    parse_quote!(
                        <#try_from as ::sqlx_d1::FromRow<#lifetime, R>>::from_row(__row)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx_d1::sqlx_core::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                }
                // Flatten + Json
                (true, _, true) => {
                    panic!("Cannot use both flatten and json")
                }
                // Try from
                (false, Some(try_from), false) => {
                    predicates
                        .push(parse_quote!(#try_from: ::sqlx_d1::sqlx_core::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(#try_from: ::sqlx_d1::sqlx_core::types::Type<R::Database>)); 

                    parse_quote!(
                        __row.try_get(#id_s)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx_d1::sqlx_core::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                }
                // Try from + Json
                (false, Some(try_from), true) => {
                    predicates
                        .push(parse_quote!(::sqlx_d1::sqlx_core::types::Json<#try_from>: ::sqlx_d1::sqlx_core::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(::sqlx_d1::sqlx_core::types::Json<#try_from>: ::sqlx_d1::sqlx_core::types::Type<R::Database>));

                    parse_quote!(
                        __row.try_get::<::sqlx_d1::sqlx_core::types::Json<_>, _>(#id_s)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v.0)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx_d1::sqlx_core::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                },
                // Json
                (false, None, true) => {
                    predicates
                        .push(parse_quote!(::sqlx_d1::sqlx_core::types::Json<#ty>: ::sqlx_d1::sqlx_core::decode::Decode<#lifetime, R::Database>));
                    predicates.push(parse_quote!(::sqlx_d1::sqlx_core::types::Json<#ty>: ::sqlx_d1::sqlx_core::types::Type<R::Database>));

                    parse_quote!(__row.try_get::<::sqlx_d1::sqlx_core::types::Json<_>, _>(#id_s).map(|x| x.0))
                },
            };

            if attributes.default {
                Some(parse_quote!(
                    let #id: #ty = #expr.or_else(|e| match e {
                        ::sqlx_d1::sqlx_core::Error::ColumnNotFound(_) => {
                            ::std::result::Result::Ok(Default::default())
                        },
                        e => ::std::result::Result::Err(e)
                    })?;
                ))
            } else if container_attributes.default {
                Some(parse_quote!(
                    let #id: #ty = #expr.or_else(|e| match e {
                        ::sqlx_d1::sqlx_core::Error::ColumnNotFound(_) => {
                            ::std::result::Result::Ok(__default.#id)
                        },
                        e => ::std::result::Result::Err(e)
                    })?;
                ))
            } else {
                Some(parse_quote!(
                    let #id: #ty = #expr?;
                ))
            }
        })
        .collect();

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let names = fields.iter().map(|field| &field.ident);

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx_d1::FromRow<#lifetime, R> for #ident #ty_generics #where_clause {
            fn from_row(__row: &#lifetime R) -> ::sqlx_d1::sqlx_core::Result<Self> {
                #default_instance

                #(#reads)*

                ::std::result::Result::Ok(#ident {
                    #(#names),*
                })
            }
        }
    ))
}

fn expand_derive_from_row_struct_unnamed(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(R: ::sqlx_d1::sqlx_core::row::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(
        ::std::primitive::usize: ::sqlx_d1::sqlx_core::column::ColumnIndex<R>
    ));

    for field in fields {
        let ty = &field.ty;

        predicates.push(parse_quote!(#ty: ::sqlx_d1::sqlx_core::decode::Decode<#lifetime, R::Database>));
        predicates.push(parse_quote!(#ty: ::sqlx_d1::sqlx_core::types::Type<R::Database>));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let gets = fields
        .iter()
        .enumerate()
        .map(|(idx, _)| quote!(row.try_get(#idx)?));

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx_d1::FromRow<#lifetime, R> for #ident #ty_generics #where_clause {
            fn from_row(row: &#lifetime R) -> ::sqlx_d1::sqlx_core::Result<Self> {
                ::std::result::Result::Ok(#ident (
                    #(#gets),*
                ))
            }
        }
    ))
}
