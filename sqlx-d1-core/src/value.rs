pub struct D1Value(worker::send::SendWrapper<worker::wasm_bindgen::JsValue>);
const _: () = {
    impl std::fmt::Debug for D1Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D1Value")
                .field("typeof", &self.0.0.js_typeof().as_string().unwrap())
                .field("value", &self.0.0)
                .finish()
        }
    }

    impl sqlx_core::value::Value for D1Value {
        type Database = crate::D1;

        fn as_ref(&self) -> <Self::Database as sqlx_core::database::Database>::ValueRef<'_> {
            D1ValueRef::from(&self.0.0)
        }

        fn type_info(
            &self,
        ) -> std::borrow::Cow<'_, <Self::Database as sqlx_core::database::Database>::TypeInfo>
        {
            std::borrow::Cow::Owned(crate::type_info::D1TypeInfo::from_raw(&self.0.0))
        }

        fn is_null(&self) -> bool {
            self.0.loose_eq(&worker::wasm_bindgen::JsValue::null())
        }
    }

    impl From<worker::wasm_bindgen::JsValue> for D1Value {
        fn from(value: worker::wasm_bindgen::JsValue) -> Self {
            Self(worker::send::SendWrapper(value))
        }
    }

    impl D1Value {
        pub(crate) fn null() -> Self {
            Self::from(worker::wasm_bindgen::JsValue::null())
        }
    }
};

pub struct D1ValueRef<'r>(worker::send::SendWrapper<&'r worker::wasm_bindgen::JsValue>);
const _: () = {
    impl<'r> sqlx_core::value::ValueRef<'r> for D1ValueRef<'r> {
        type Database = crate::D1;

        fn to_owned(&self) -> <Self::Database as sqlx_core::database::Database>::Value {
            D1Value::from(self.0.0.clone())
        }

        fn type_info(
            &self,
        ) -> std::borrow::Cow<'_, <Self::Database as sqlx_core::database::Database>::TypeInfo>
        {
            std::borrow::Cow::Owned(crate::type_info::D1TypeInfo::from_raw(self.0.0))
        }

        fn is_null(&self) -> bool {
            self.0.loose_eq(&worker::wasm_bindgen::JsValue::null())
        }
    }

    impl<'r> std::fmt::Debug for D1ValueRef<'r> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <D1Value as std::fmt::Debug>::fmt(&sqlx_core::value::ValueRef::to_owned(self), f)
        }
    }

    impl<'r> std::ops::Deref for D1ValueRef<'r> {
        type Target = worker::wasm_bindgen::JsValue;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'r> From<D1ValueRef<'r>> for worker::wasm_bindgen::JsValue  {
        fn from(d1valueref: D1ValueRef<'r>) -> worker::wasm_bindgen::JsValue {
            d1valueref.0.0.clone()
        }
    }

    impl<'r> From<&'r worker::wasm_bindgen::JsValue> for D1ValueRef<'r> {
        fn from(value: &'r worker::wasm_bindgen::JsValue) -> Self {
            Self(worker::send::SendWrapper(value))
        }
    }
};
