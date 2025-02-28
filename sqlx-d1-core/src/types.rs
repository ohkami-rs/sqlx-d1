use crate::{D1, type_info::D1TypeInfo, value::D1Value, error::D1Error};
use sqlx_core::encode::{IsNull, Encode};
use sqlx_core::decode::Decode;
use sqlx_core::types::Type;
use worker::{serde_wasm_bindgen, wasm_bindgen::JsValue};

impl<'q, E: Encode<'q, D1>> Encode<'q, D1> for Option<E> {
    fn encode_by_ref(
        &self,
        buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
        match self {
            Some(e) => {
                <E as Encode<'q, D1>>::encode_by_ref(e, buf)
            }
            None => {
                buf.push(D1Value::null());
                Ok(IsNull::Yes)
            }
        }
    }
}

macro_rules! serialize {
    ($q:lifetime) => {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<$q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            buf.push(D1Value::from(serde_wasm_bindgen::to_value(self).map_err(D1Error::from_rust)?));
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
            D1TypeInfo::boolean()
        }
    }

    impl<'q> Encode<'q, D1> for bool {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            buf.push(D1Value::from(JsValue::from_f64(if *self {1.} else {0.})));
            Ok(IsNull::No)
        }
    }

    impl Decode<'_, D1> for bool {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok((&*value).as_f64().is_some_and(|n| n != 0.))
        }
    }
};

/// ref: <https://github.com/launchbadge/sqlx/blob/d4ae6ffd882ed2de1695c652888d809bc068554e/sqlx-sqlite/src/types/text.rs>
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

#[cfg(feature = "json")]
/// ref: <https://github.com/launchbadge/sqlx/blob/d4ae6ffd882ed2de1695c652888d809bc068554e/sqlx-sqlite/src/types/json.rs>
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

#[cfg(feature = "uuid")]
/// ref: <https://github.com/launchbadge/sqlx/blob/d4ae6ffd882ed2de1695c652888d809bc068554e/sqlx-sqlite/src/types/uuid.rs>
const _: (/* UUID */) = {
    use sqlx_core::types::uuid::{Uuid, fmt::{Hyphenated, Simple}};

    impl Type<D1> for Uuid {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <Vec<u8> as Type<D1>>::type_info()
        }
    }
    impl<'q> Encode<'q, D1> for Uuid {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <Vec<u8> as Encode<'q, D1>>::encode(self.into_bytes().into(), buf)
        }
    }
    impl Decode<'_, D1> for Uuid {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok(Uuid::from_slice(&<Vec<u8> as Decode<D1>>::decode(value)?).map_err(D1Error::from_rust)?)
        }
    }

    impl Type<D1> for Hyphenated {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <String as Type<D1>>::type_info()
        }
    }
    impl<'q> Encode<'q, D1> for Hyphenated {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'q, D1>>::encode(self.to_string(), buf)            
        }
    }
    impl Decode<'_, D1> for Hyphenated {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let uuid = Uuid::parse_str(&<String as Decode<D1>>::decode(value)?).map_err(D1Error::from_rust)?;
            Ok(uuid.hyphenated())
        }
    }

    impl Type<D1> for Simple {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <String as Type<D1>>::type_info()
        }
    }
    impl<'q> Encode<'q, D1> for Simple {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'q, D1>>::encode(self.to_string(), buf)            
        }
    }
    impl Decode<'_, D1> for Simple {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let uuid = Uuid::parse_str(&<String as Decode<D1>>::decode(value)?).map_err(D1Error::from_rust)?;
            Ok(uuid.simple())
        }
    }
};

/* ref: <https://github.com/launchbadge/sqlx/blob/277dd36c7868acb10eae20f50418e273b71c8499/sqlx-sqlite/src/type_checking.rs> */
sqlx_core::impl_type_checking! {
    crate::D1 {
        bool,
        i64,
        f64,
        String,
        Vec<u8>,

        #[cfg(feature = "uuid")]
        sqlx_core::types::Uuid,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
}
