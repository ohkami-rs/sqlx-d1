#[derive(Debug)]
pub struct D1Column {
    pub(crate) ordinal: usize,
    pub(crate) name: sqlx_core::ext::ustr::UStr,
    // pub(crate) value: crate::value::D1Value,
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
