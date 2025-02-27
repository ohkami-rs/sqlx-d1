#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "macro-support", derive(serde::Serialize, serde::Deserialize))]
pub struct D1TypeInfo(D1Type);

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "macro-support", derive(serde::Serialize, serde::Deserialize))]
enum D1Type {
    Null,
    Real,
    Integer,
    Text,
    Blob,
}

impl std::fmt::Display for D1TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl sqlx_core::type_info::TypeInfo for D1TypeInfo {
    fn is_null(&self) -> bool {
        matches!(self.0, D1Type::Null)
    }
    
    fn name(&self) -> &str {
        match self.0 {
            D1Type::Null => "NULL",
            D1Type::Text => "TEXT",
            D1Type::Real => "REAL",
            D1Type::Blob => "BLOB",
            D1Type::Integer => "INTEGER"
        }
    }
}

impl D1TypeInfo {
    pub(crate) fn unknown() -> &'static Self {
        /* most least-bad choice */
        &Self(D1Type::Blob)
    }

    pub const fn null() -> Self {
        Self(D1Type::Null)
    }
    pub const fn real() -> Self {
        Self(D1Type::Real)
    }
    pub const fn integer() -> Self {
        Self(D1Type::Integer)
    }
    pub const fn text() -> Self {
        Self(D1Type::Text)
    }
    pub const fn blob() -> Self {
        Self(D1Type::Blob)
    }

    pub(crate) fn from_raw(raw: &worker::wasm_bindgen::JsValue) -> Self {
        if raw.is_null() || raw.is_undefined() {
            Self::null()
        } else if raw.is_string() {
            Self::text()
        } else if raw.as_bool().is_some() {
            Self::integer()
        } else if raw.as_f64().is_some() {
            if worker::js_sys::Number::is_safe_integer(raw) {
                Self::integer()
            } else {
                Self::real()
            }
        } else {
            *Self::unknown()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl D1TypeInfo {
    pub(crate) fn from_sqlite(sqlite_type_info: sqlx_sqlite::SqliteTypeInfo) -> Self {
        use sqlx_core::type_info::TypeInfo as _;

        /* ref: <https://github.com/launchbadge/sqlx/blob/25efb2f7f410e0f0aa3fee1d8467429066dbcdf8/sqlx-sqlite/src/type_info.rs#L56-L71> */
        match sqlite_type_info.name() {
            "NULL" => Self::null(),
            "TEXT" => Self::text(),
            "REAL" => Self::real(),
            "BLOB" => Self::blob(),
            "INTEGER" | "NUMERIC" | "BOOLEAN" => Self::integer(),
            _ => *Self::unknown()
        }
    }
}
