use crate::column::D1Column;

pub struct D1Row(
    Vec<D1Column>
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

impl D1Row {
    pub(crate) fn from_raw(raw: worker::wasm_bindgen::JsValue) -> Result<Self, sqlx_core::Error> {
        use worker::wasm_bindgen::JsCast;
        use worker::js_sys::{Object, Array};

        let entries = Object::entries(raw.unchecked_ref::<Object>());

        let mut columns = Vec::with_capacity(entries.length() as usize);
        for (i, pair) in entries.iter().enumerate() {
            let pair = pair.unchecked_into::<Array>();
            let (key, value) = (pair.get(0), pair.get(1));
            columns.push(D1Column {
                ordinal: i,
                name: key.as_string().unwrap().into(),
                value: value.into(),
            });
        }

        Ok(Self(columns))
    }
}
