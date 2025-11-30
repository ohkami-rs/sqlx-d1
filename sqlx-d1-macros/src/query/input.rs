//! ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/query/input.rs>

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, ExprArray, LitBool, LitStr, Token, Type};

/// Macro input shared by `query!()` and `query_file!()`
pub struct QueryMacroInput {
    pub(super) sql: String,

    pub(super) src_span: Span,

    pub(super) record_type: RecordType,

    pub(super) arg_exprs: Vec<Expr>,

    pub(super) checked: bool,

    pub(super) file_path: Option<String>,
}

enum QuerySrc {
    String(String),
    File(String),
}

pub enum RecordType {
    Given(Type),
    Scalar,
    Generated,
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut query_src: Option<(QuerySrc, Span)> = None;
        let mut args: Option<Vec<Expr>> = None;
        let mut record_type = RecordType::Generated;
        let mut checked = true;

        let mut expect_comma = false;

        while !input.is_empty() {
            if expect_comma {
                let _ = input.parse::<syn::token::Comma>()?;
            }

            let key: Ident = input.parse()?;

            let _ = input.parse::<syn::token::Eq>()?;

            if key == "source" {
                let span = input.span();
                let query_str = Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
                    .iter()
                    .map(LitStr::value)
                    .collect();
                query_src = Some((QuerySrc::String(query_str), span));
            } else if key == "source_file" {
                let lit_str = input.parse::<LitStr>()?;
                query_src = Some((QuerySrc::File(lit_str.value()), lit_str.span()));
            } else if key == "args" {
                let exprs = input.parse::<ExprArray>()?;
                args = Some(exprs.elems.into_iter().collect())
            } else if key == "record" {
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                record_type = RecordType::Given(input.parse()?);
            } else if key == "scalar" {
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }

                // we currently expect only `scalar = _`
                // a `query_as_scalar!()` variant seems less useful than just overriding the type
                // of the column in SQL
                input.parse::<syn::Token![_]>()?;
                record_type = RecordType::Scalar;
            } else if key == "checked" {
                let lit_bool = input.parse::<LitBool>()?;
                checked = lit_bool.value;
            } else {
                let message = format!("unexpected input key: {key}");
                return Err(syn::Error::new_spanned(key, message));
            }

            expect_comma = true;
        }

        let (src, src_span) =
            query_src.ok_or_else(|| input.error("expected `source` or `source_file` key"))?;

        let arg_exprs = args.unwrap_or_default();

        let file_path = src.file_path(src_span)?;

        Ok(QueryMacroInput {
            sql: src.resolve(src_span)?,
            src_span,
            record_type,
            arg_exprs,
            checked,
            file_path,
        })
    }
}

impl QueryMacroInput {
    /// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/query/args.rs>
    pub(super) fn quote_args_with(
        &self,
        describe: &sqlx_core::describe::Describe<sqlx_d1_core::D1>,
    ) -> syn::Result<TokenStream> {
        if self.arg_exprs.is_empty() {
            return Ok(quote! {
                let query_args = ::core::result::Result::<_, ::sqlx_d1::sqlx_core::error::BoxDynError>::Ok(
                    <::sqlx_d1::D1 as ::sqlx_d1::sqlx_core::database::Database>::Arguments::<'_>::default()
                );
            });
        }

        let arg_idents = (0..self.arg_exprs.len())
            .map(|i| format_ident!("arg{i}"))
            .collect::<Vec<_>>();
        let arg_exprs = self.arg_exprs.iter().cloned().map(strip_wildcard);

        let arg_bindings = quote! {
            #(let #arg_idents = &(#arg_exprs);)*
        };

        let args_check = match describe.parameters() {
            None | Some(sqlx_core::Either::Right(_)) => TokenStream::new(),

            Some(sqlx_core::Either::Left(_)) if !self.checked => TokenStream::new(),

            Some(sqlx_core::Either::Left(params)) => params
                .iter()
                .zip(arg_idents.iter().zip(&self.arg_exprs))
                .enumerate()
                .map(|(i, (param_type_info, (param_name, param_expr)))| {
                    if get_type_override(param_expr).is_some() {
                        return Ok(TokenStream::new());
                    }

                    let param_type_name = <sqlx_d1_core::D1 as sqlx_core::type_checking::TypeChecking>::param_type_for_id(&param_type_info)
                        .ok_or_else(|| syn::Error::new(
                            param_expr.span(),
                            format!("unsupported type {param_type_info} for param #{}", i + 1)
                        ))?
                        .parse::<TokenStream>()
                        .map_err(|_| syn::Error::new(
                            param_expr.span(),
                            format!("Rust yupe mapping for {param_type_info} not parsable")
                        ))?;

                    syn::Result::Ok(quote_spanned!(param_expr.span() => {
                        if false {
                            use ::sqlx_d1::sqlx_core::ty_match::{self, WrapSame, WrapSameExt, MatchBorrow, MatchBorrowExt};

                            let expr = ty_match::dupe_value(#param_name);
                            let ty_check = WrapSame::<#param_type_name, _>::new(&expr).wrap_same();
                            let (mut _ty_check, match_borrow) = MatchBorrow::new(ty_check, &expr);
                            _ty_check = match_borrow.match_borrow();

                            ::std::panic!()
                        }
                    }))
                })
                .collect::<syn::Result<TokenStream>>()?
        };

        let args_count = self.arg_exprs.len();

        Ok(quote! {
            #arg_bindings

            #args_check

            let mut query_args = <::sqlx_d1::D1 as ::sqlx_d1::sqlx_core::database::Database>::Arguments::<'_>::default();
            query_args.reserve(#args_count, 0 #(+ ::sqlx_d1::sqlx_core::encode::Encode::<::sqlx_d1::D1>::size_hint(#arg_idents))*);

            let query_args = ::core::result::Result::<_, ::sqlx_d1::sqlx_core::error::BoxDynError>::Ok(query_args)
            #( .and_then(move |mut query_args| {query_args.add(#arg_idents)?; Ok(query_args)}) )*;
        })
    }
}

impl QuerySrc {
    /// If the query source is a file, read it to a string. Otherwise return the query string.
    fn resolve(self, source_span: Span) -> syn::Result<String> {
        match self {
            QuerySrc::String(string) => Ok(string),
            QuerySrc::File(file) => read_file_src(&file, source_span),
        }
    }

    fn file_path(&self, source_span: Span) -> syn::Result<Option<String>> {
        if let QuerySrc::File(ref file) = *self {
            let path = resolve_path(file, source_span)?
                .canonicalize()
                .map_err(|e| syn::Error::new(source_span, e))?;

            Ok(Some(
                path.to_str()
                    .ok_or_else(|| {
                        syn::Error::new(
                            source_span,
                            "query file path cannot be represented as a string",
                        )
                    })?
                    .to_string(),
            ))
        } else {
            Ok(None)
        }
    }
}

fn read_file_src(source: &str, source_span: Span) -> syn::Result<String> {
    let file_path = resolve_path(source, source_span)?;

    fs::read_to_string(&file_path).map_err(|e| {
        syn::Error::new(
            source_span,
            format!(
                "failed to read query file at {}: {}",
                file_path.display(),
                e
            ),
        )
    })
}

/// ref: <https://github.com/launchbadge/sqlx/blob/1c7b3d0751cdca5a08fbfa7f24c985fc3774cf11/sqlx-macros-core/src/common.rs>
fn resolve_path(path: impl AsRef<Path>, err_span: Span) -> syn::Result<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        return Err(syn::Error::new(
            err_span,
            "absolute paths will only work on the current machine",
        ));
    }

    // requires `proc_macro::SourceFile::path()` to be stable
    // https://github.com/rust-lang/rust/issues/54725
    if path.is_relative()
        && !path
            .parent()
            .map_or(false, |parent| !parent.as_os_str().is_empty())
    {
        return Err(syn::Error::new(
            err_span,
            "paths relative to the current file's directory are not currently supported",
        ));
    }

    Ok(super::LOCATION.manifest_dir.join(path))
}

fn strip_wildcard(expr: Expr) -> Expr {
    use syn::{ExprCast, ExprGroup};

    match expr {
        Expr::Group(ExprGroup {
            attrs,
            group_token,
            expr,
        }) => Expr::Group(ExprGroup {
            attrs,
            group_token,
            expr: Box::new(strip_wildcard(*expr)),
        }),
        // we want to retain casts if they semantically matter
        Expr::Cast(ExprCast {
            attrs,
            expr,
            as_token,
            ty,
        }) => match *ty {
            // cast to wildcard `_` will produce weird errors; we interpret it as taking the value as-is
            Type::Infer(_) => *expr,
            _ => Expr::Cast(ExprCast {
                attrs,
                expr,
                as_token,
                ty,
            }),
        },
        _ => expr,
    }
}

fn get_type_override(expr: &Expr) -> Option<&Type> {
    match expr {
        Expr::Group(group) => get_type_override(&group.expr),
        Expr::Cast(cast) => Some(&cast.ty),
        _ => None,
    }
}
