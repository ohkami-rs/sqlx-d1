#[derive(Debug)]
pub struct D1Column {
    name: sqlx_core::ext::ustr::UStr,
    ordinal: usize,
    type_info: crate::type_info::D1TypeInfo,
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
        &self.type_info
    }
}
