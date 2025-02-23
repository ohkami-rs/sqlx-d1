#[derive(Debug)]
pub struct D1Column {
    ordinal: usize,
    name: sqlx_core::ext::ustr::UStr,
    value: crate::value::D1Value,
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

impl D1Column {
    pub(crate) fn value_ref(&self) -> crate::value::D1ValueRef<'_> {
        sqlx_core::value::Value::as_ref(&self.value)
    }
}
