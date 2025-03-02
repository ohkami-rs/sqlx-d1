//! ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/derives/attributes.rs>

use syn::{Attribute, LitStr, Token, Type};

macro_rules! fail {
    ($t:expr, $m:expr) => {
        return Err(syn::Error::new_spanned($t, $m))
    };
}

macro_rules! try_set {
    ($i:ident, $v:expr, $t:expr) => {
        match $i {
            None => $i = Some($v),
            Some(_) => fail!($t, "duplicate attribute"),
        }
    };
}

/*// not used in `#[derive(FromRow)]`
pub struct TypeName {
    pub val: String,
    pub span: Span,
}

impl TypeName {
    pub fn get(&self) -> TokenStream {
        let val = &self.val;
        quote_spanned! { self.span => #val }
    }
}
*/

#[derive(Copy, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum RenameAll {
    LowerCase,
    SnakeCase,
    UpperCase,
    ScreamingSnakeCase,
    KebabCase,
    CamelCase,
    PascalCase,
}
impl RenameAll {
    /// ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/sqlx-macros-core/src/derives/mod.rs#L27-L37>
    pub(super) fn apply(self, s: &str) -> String {
        use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

        match self {
            RenameAll::LowerCase => s.to_lowercase(),
            RenameAll::SnakeCase => s.to_snake_case(),
            RenameAll::UpperCase => s.to_uppercase(),
            RenameAll::ScreamingSnakeCase => s.to_shouty_snake_case(),
            RenameAll::KebabCase => s.to_kebab_case(),
            RenameAll::CamelCase => s.to_lower_camel_case(),
            RenameAll::PascalCase => s.to_upper_camel_case(),
        }
    }
}

pub struct SqlxContainerAttributes {
    /*// not used in `#[derive(FromRow)]`
        pub transparent: bool,
        pub type_name: Option<TypeName>,
        pub repr: Option<Ident>,
        pub no_pg_array: bool,
    */
    pub rename_all: Option<RenameAll>,
    pub default: bool,
}

pub struct SqlxChildAttributes {
    pub rename: Option<String>,
    pub default: bool,
    pub flatten: bool,
    pub try_from: Option<Type>,
    pub skip: bool,
    pub json: bool,
}

pub fn parse_container_attributes(input: &[Attribute]) -> syn::Result<SqlxContainerAttributes> {
    /*// not used in `#[derive(FromRow)]`
        let mut transparent = None;
        let mut repr = None;
        let mut type_name = None;
        let mut no_pg_array = None;
    */
    let mut rename_all = None;
    let mut default = None;

    for attr in input {
        if attr.path().is_ident("sqlx") {
            attr.parse_nested_meta(|meta| {
                /*// not used in `#[derive(FromRow)]`
                    if meta.path.is_ident("transparent") {
                        try_set!(transparent, true, attr);
                    } else if meta.path.is_ident("type_name") {
                        meta.input.parse::<Token![=]>()?;
                        let lit: LitStr = meta.input.parse()?;
                        let name = TypeName {
                            val: lit.value(),
                            span: lit.span(),
                        };

                        try_set!(type_name, name, lit)
                    } else if meta.path.is_ident("no_pg_array") {
                        try_set!(no_pg_array, true, attr);
                    } else
                */
                if meta.path.is_ident("rename_all") {
                    meta.input.parse::<Token![=]>()?;
                    let lit: LitStr = meta.input.parse()?;

                    let val = match lit.value().as_str() {
                        "lowercase" => RenameAll::LowerCase,
                        "snake_case" => RenameAll::SnakeCase,
                        "UPPERCASE" => RenameAll::UpperCase,
                        "SCREAMING_SNAKE_CASE" => RenameAll::ScreamingSnakeCase,
                        "kebab-case" => RenameAll::KebabCase,
                        "camelCase" => RenameAll::CamelCase,
                        "PascalCase" => RenameAll::PascalCase,
                        _ => fail!(lit, "unexpected value for rename_all"),
                    };

                    try_set!(rename_all, val, lit)
                } else if meta.path.is_ident("default") {
                    try_set!(default, true, attr);
                } else {
                    fail!(meta.path, "unexpected attribute")
                }

                Ok(())
            })?;
        }
        /*// not used in `#[derive(FromRow)]`
            else if attr.path().is_ident("repr") {
                let list: Punctuated<Meta, Token![,]> =
                    attr.parse_args_with(<Punctuated<Meta, Token![,]>>::parse_terminated)?;

                if let Some(path) = list.iter().find_map(|f| f.require_path_only().ok()) {
                    try_set!(repr, path.get_ident().unwrap().clone(), list);
                }
            }
        */
    }

    Ok(SqlxContainerAttributes {
        /*// not used in `#[derive(FromRow)]`
            transparent: transparent.unwrap_or(false),
            repr,
            type_name,
            no_pg_array: no_pg_array.unwrap_or(false),
        */
        rename_all,
        default: default.unwrap_or(false),
    })
}

pub fn parse_child_attributes(input: &[Attribute]) -> syn::Result<SqlxChildAttributes> {
    let mut rename = None;
    let mut default = false;
    let mut try_from = None;
    let mut flatten = false;
    let mut skip: bool = false;
    let mut json = false;

    for attr in input.iter().filter(|a| a.path().is_ident("sqlx")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                meta.input.parse::<Token![=]>()?;
                let val: LitStr = meta.input.parse()?;
                try_set!(rename, val.value(), val);
            } else if meta.path.is_ident("try_from") {
                meta.input.parse::<Token![=]>()?;
                let val: LitStr = meta.input.parse()?;
                try_set!(try_from, val.parse()?, val);
            } else if meta.path.is_ident("default") {
                default = true;
            } else if meta.path.is_ident("flatten") {
                flatten = true;
            } else if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("json") {
                json = true;
            }

            Ok(())
        })?;

        if json && flatten {
            fail!(
                attr,
                "Cannot use `json` and `flatten` together on the same field"
            );
        }
    }

    Ok(SqlxChildAttributes {
        rename,
        default,
        flatten,
        try_from,
        skip,
        json,
    })
}
