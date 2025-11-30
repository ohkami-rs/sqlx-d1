#[derive(Default)]
pub struct D1Arguments(Vec<crate::value::D1Value>);

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
        T: 'q
            + sqlx_core::encode::Encode<'q, Self::Database>
            + sqlx_core::types::Type<Self::Database>,
    {
        let len_before_encode = self.0.len();
        let _/* IsNull */ = value.encode(&mut self.0)
            .inspect_err(|_| self.0.truncate(len_before_encode))?;
        Ok(())
    }
}

impl<'q> sqlx_core::arguments::IntoArguments<'q, crate::D1> for D1Arguments {
    fn into_arguments(self) -> <crate::D1 as sqlx_core::database::Database>::Arguments<'q> {
        self
    }
}

impl AsRef<[worker::wasm_bindgen::JsValue]> for D1Arguments {
    fn as_ref(&self) -> &[worker::wasm_bindgen::JsValue] {
        let this: &[crate::value::D1Value] = self.0.as_slice();

        /* SAFETY: `D1Value` is newtype of `JsValue` */
        unsafe { std::mem::transmute(this) }
    }
}
