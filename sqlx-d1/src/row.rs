use crate::{column::D1Column, value::D1Value};
use sqlx_core::value::Value;

pub struct D1Row {
    columns: Vec<D1Column>,
    values:  Vec<D1Value>,
}

impl sqlx_core::row::Row for D1Row {
    type Database = crate::D1;

    fn columns(&self) -> &[<Self::Database as sqlx_core::database::Database>::Column] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<<Self::Database as sqlx_core::database::Database>::ValueRef<'_>, sqlx_core::Error>
    where
        I: sqlx_core::column::ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(self.values[index].as_ref())
    }
}

impl D1Row {
    pub(crate) fn from_raw(raw: worker::wasm_bindgen::JsValue) -> Result<Self, sqlx_core::Error> {
        use worker::wasm_bindgen::JsCast;
        use worker::js_sys::{Object, Array};

        let entries = Object::entries(raw.unchecked_ref::<Object>());
        let len = entries.length() as usize;

        let (mut columns, mut values) = (Vec::with_capacity(len), Vec::with_capacity(len));
        for (i, pair) in entries.iter().enumerate() {
            let pair = pair.unchecked_into::<Array>();
            let (key, value) = (pair.get(0), pair.get(1));
            columns.push(D1Column {
                ordinal: i,
                name: key.as_string().unwrap().into(),
            });
            values.push(
                value.into()
            );
        }

        Ok(Self { columns, values })
    }
}
