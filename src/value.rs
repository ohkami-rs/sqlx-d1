pub struct D1Value(worker::wasm_bindgen::JsValue);

impl sqlx_core::value::Value for D1Value {
    type Database = crate::D1;

    fn as_ref(&self) -> <Self::Database as sqlx_core::database::Database>::ValueRef<'_> {
        D1ValueRef(&self.0)
    }

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as sqlx_core::database::Database>::TypeInfo> {
        std::borrow::Cow::Owned(crate::type_info::D1TypeInfo::unknown())
    }

    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

pub struct D1ValueRef<'r>(&'r worker::wasm_bindgen::JsValue);

impl<'r> sqlx_core::value::ValueRef<'r> for D1ValueRef<'r> {
    type Database = crate::D1;

    fn to_owned(&self) -> <Self::Database as sqlx_core::database::Database>::Value {
        D1Value(self.0.clone())
    }

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as sqlx_core::database::Database>::TypeInfo> {
        std::borrow::Cow::Owned(crate::type_info::D1TypeInfo::unknown())
    }

    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
