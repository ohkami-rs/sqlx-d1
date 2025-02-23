use crate::{D1, type_info::D1TypeInfo, arguments::D1ArgumentValue, error::D1Error};
use sqlx_core::encode::{IsNull, Encode};
use sqlx_core::decode::Decode;
use sqlx_core::types::Type;
use worker::{serde_wasm_bindgen, wasm_bindgen::JsValue};

macro_rules! serialize {
    ($q:lifetime) => {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<$q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            buf.push(D1ArgumentValue::from(serde_wasm_bindgen::to_value(self).map_err(D1Error::from_rust)?));
            Ok(IsNull::No)            
        }
    };
}

macro_rules! deserialize {
    () => {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok(serde_wasm_bindgen::from_value(value.into()).map_err(D1Error::from_rust)?)
        }
    };
}

macro_rules! serde_wasm_bindgen {
    ($T:ty as $d1_type:ident) => {
        impl Type<D1> for $T {
            fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
                D1TypeInfo::$d1_type()
            }
        }

        impl<'q> Encode<'q, D1> for $T {
            serialize!('q);
        }

        impl Decode<'_, D1> for $T {
            deserialize!();
        }
    };
}

impl Type<D1> for [u8] {
    fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
        D1TypeInfo::blob()
    }
}
impl<'q> Encode<'q, D1> for &'q [u8] {
    serialize!('q);
}
serde_wasm_bindgen!(Box<[u8]> as blob);
serde_wasm_bindgen!(Vec<u8> as blob);

serde_wasm_bindgen!(f32 as real);
serde_wasm_bindgen!(f64 as real);

serde_wasm_bindgen!(i8 as integer);
serde_wasm_bindgen!(i16 as integer);
serde_wasm_bindgen!(i32 as integer);
serde_wasm_bindgen!(i64 as integer);
serde_wasm_bindgen!(isize as integer);

serde_wasm_bindgen!(u8 as integer);
serde_wasm_bindgen!(u16 as integer);
serde_wasm_bindgen!(u32 as integer);
serde_wasm_bindgen!(u64 as integer);
serde_wasm_bindgen!(usize as integer);

impl Type<D1> for str {
    fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
        D1TypeInfo::blob()
    }
}
impl<'q> Encode<'q, D1> for &'q str {
    serialize!('q);
}
serde_wasm_bindgen!(Box<str> as text);
serde_wasm_bindgen!(String as text);
serde_wasm_bindgen!(std::borrow::Cow<'_, str> as text);

/// specialized conversion: true <-> 1 / false <-> 0
const _: (/* bool */) = {
    impl Type<D1> for bool {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            D1TypeInfo::integer()
        }
    }

    impl<'q> Encode<'q, D1> for bool {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            buf.push(D1ArgumentValue::from(JsValue::from_f64(if *self {1.} else {0.})));
            Ok(IsNull::No)
        }
    }

    impl Decode<'_, D1> for bool {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok((&*value).as_f64().is_some_and(|n| n != 0.))
        }
    }
};

const _: (/* generics text */) = {
    use sqlx_core::types::Text;

    impl<T> Type<D1> for Text<T> {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <String as Type<D1>>::type_info()
        }
    }

    impl<'q, T> Encode<'q, D1> for Text<T>
    where
        T: std::fmt::Display,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'q, D1>>::encode(self.0.to_string(), buf)
        }
    }

    impl<T> Decode<'_, D1> for Text<T>
    where
        T: std::str::FromStr,
        sqlx_core::error::BoxDynError: From<<T as std::str::FromStr>::Err>,
    {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok(Self(<String as Decode<D1>>::decode(value)?.parse()?))
        }
    }
};

#[cfg(feature="json")]
const _: (/* generic JSON */) = {
    use sqlx_core::types::Json;

    impl<T> Type<D1> for Json<T> {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <String as Type<D1>>::type_info()
        }
    }

    impl<'q, T> Encode<'q, D1> for Json<T>
    where
        T: serde::Serialize,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'q, D1>>::encode(self.encode_to_string()?, buf)
        }
    }

    impl<T> Decode<'_, D1> for Json<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Self::decode_from_string(&<String as Decode<D1>>::decode(value)?)
        }
    }
};

#[cfg(feature="uuid")]
const _: (/* UUID */) = {
    use sqlx_core::types::Uuid;

    
};
