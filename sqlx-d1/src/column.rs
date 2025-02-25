#[derive(Debug)]
pub struct D1Column {
    pub(crate) ordinal: usize,
    pub(crate) name: sqlx_core::ext::ustr::UStr,
}

impl sqlx_core::column::Column for D1Column {
    type Database = crate::D1;

    fn name(&self) -> &str {
        &self.name
    }

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn type_info(&self) -> &<Self::Database as sqlx_core::database::Database>::TypeInfo {
        crate::type_info::D1TypeInfo::unknown()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl D1Column {
    pub(crate) fn from_sqlite(sqlite_column: sqlx_sqlite::SqliteColumn) -> Self {
        use sqlx_core::column::Column as _;

        Self {
            ordinal: sqlite_column.ordinal(),
            name: sqlite_column.name().to_string().into(),
        }
    }
}
