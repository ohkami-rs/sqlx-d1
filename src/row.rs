pub struct D1Row(
    Vec<crate::column::D1Column>
);

impl sqlx_core::row::Row for D1Row {
    type Database = crate::D1;

    fn columns(&self) -> &[<Self::Database as sqlx_core::database::Database>::Column] {
        &self.0
    }

    fn try_get_raw<I>(&self, index: I) -> Result<<Self::Database as sqlx_core::database::Database>::ValueRef<'_>, sqlx_core::Error>
    where
        I: sqlx_core::column::ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(self.0[index].value_ref())
    }
}
