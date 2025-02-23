pub struct D1Row(worker::send::SendWrapper<
    Vec<worker::wasm_bindgen::JsValue>
>);

impl sqlx_core::row::Row for D1Row {
    type Database = crate::D1;

    fn columns(&self) -> &[<Self::Database as sqlx_core::database::Database>::Column] {
        
    }
}
