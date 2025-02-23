pub struct D1ArgumentValue(
    crate::value::D1Value
);

#[derive(Default)]
pub struct D1Arguments(
    Vec<D1ArgumentValue>
);

impl<'q> sqlx_core::arguments::Arguments<'q> for D1Arguments {
    type Database = crate::D1;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn reserve(&mut self, additional: usize, _size_hint: usize) {
        self.0.reserve(additional);
    }

    fn add<T>(&mut self, value: T) -> Result<(), sqlx_core::error::BoxDynError>
    where
        T: 'q + sqlx_core::encode::Encode<'q, Self::Database> + sqlx_core::types::Type<Self::Database>,
    {
        let len_before_encode = self.0.len();

        match value.encode(&mut self.0) {
            Ok(sqlx_core::encode::IsNull::No) => (),
            Ok(sqlx_core::encode::IsNull::Yes) => {
                self.0.push(D1ArgumentValue(crate::value::D1Value::null()));
            }
            Err(e) => {
                self.0.truncate(len_before_encode);
                return Err(e);
            }
        }

        Ok(())    
    }
}

impl<'q> From<worker::wasm_bindgen::JsValue> for D1ArgumentValue {
    fn from(value: worker::wasm_bindgen::JsValue) -> Self {
        Self(crate::value::D1Value::from(value))
    }
}
