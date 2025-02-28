//! ref: <https://github.com/launchbadge/sqlx/blob/277dd36c7868acb10eae20f50418e273b71c8499/sqlx-sqlite/src/types/chrono.rs>

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
const _: (/* json */) = {
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
const _: (/* uuid */) = {
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

#[cfg(feature = "chrono")]
/// ref: <https://github.com/launchbadge/sqlx/blob/277dd36c7868acb10eae20f50418e273b71c8499/sqlx-sqlite/src/types/chrono.rs>
const _: (/* chrono */) = {
    use sqlx_core::types::chrono::{
        FixedOffset,
        DateTime,
        Local,
        NaiveDate,
        NaiveTime,
        NaiveDateTime,
        TimeZone,
        Utc,
    };

    impl<Tz: TimeZone> Type<D1> for DateTime<Tz> {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            <NaiveDateTime as Type<D1>>::type_info()
        }
        fn compatible(ty: &<D1 as sqlx_core::database::Database>::TypeInfo) -> bool {
            <NaiveDateTime as Type<D1>>::compatible(ty)
        }
    }
    impl<Tz: TimeZone> Encode<'_, D1> for DateTime<Tz> {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'_>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            let mut rfc3339 = self.to_rfc3339();
            if rfc3339.ends_with('Z') {let _ = rfc3339.pop().unwrap();}
            <String as Encode<'_, D1>>::encode(rfc3339, buf)
        }
    }
    impl Decode<'_, D1> for DateTime<Utc> {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let fixed_offset = <DateTime<FixedOffset> as Decode<'_, D1>>::decode(value)?;
            Ok(Utc.from_utc_datetime(&fixed_offset.naive_utc()))
        }
    }
    impl Decode<'_, D1> for DateTime<Local> {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let fixed_offset = <DateTime<FixedOffset> as Decode<'_, D1>>::decode(value)?;
            Ok(Local.from_utc_datetime(&fixed_offset.naive_utc()))
        }
    }
    impl Decode<'_, D1> for DateTime<FixedOffset> {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            return decode_or_none(&value).ok_or_else(|| From::from(format!(
                "expected datetime but got unparsable `{value:?}`"
            )));

            fn decode_or_none(
                value: &<D1 as sqlx_core::database::Database>::ValueRef<'_>
            ) -> Option<DateTime<FixedOffset>> {
                use {sqlx_core::value::ValueRef, crate::type_info::D1Type::*};

                macro_rules! return_some_if_ok {
                    ($result:expr) => {
                        if let Ok(it) = $result {
                            return Some(it);
                        }
                    };
                    ($result:expr => |$v:ident| $conv:expr) => {
                        if let Ok(it) = $result {
                            return Some((|$v| $conv)(it));
                        }
                    };
                }
                
                match &**value.type_info() {
                    Text => {
                        let value = value.as_string()?;
                        return_some_if_ok!(DateTime::parse_from_rfc3339(&value));
                        for format in &[
                            "%F %T%.f",
                            "%F %R",
                            "%F %RZ",
                            "%F %R%:z",
                            "%F %T%.fZ",
                            "%F %T%.f%:z",
                            "%FT%R",
                            "%FT%RZ",
                            "%FT%R%:z",
                            "%FT%T%.f",
                            "%FT%T%.fZ",
                            "%FT%T%.f%:z",
                        ] {
                            return_some_if_ok!(DateTime::parse_from_str(&value, format));
                            return_some_if_ok!(NaiveDateTime::parse_from_str(&value, format)
                                => |it| FixedOffset::east_opt(0).unwrap().from_utc_datetime(&it));
                        }
                        None
                    }
                    Integer => {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let value = value.as_f64()? as i64;
                        FixedOffset::east_opt(0).unwrap().timestamp_opt(value, 0).single()
                    }
                    Real => {
                        let value = value.as_f64()?;

                        let epoch_in_julian_days = 2_440_587.5;
                        let seconds_in_day = 86400.0;
                        let timestamp = (value - epoch_in_julian_days) * seconds_in_day;
                    
                        if !timestamp.is_finite() {
                            return None;
                        }
                    
                        // We don't really have a choice but to do lossy casts for this conversion
                        // We checked above if the value is infinite or NaN which could otherwise cause problems
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        {
                            let seconds = timestamp.trunc() as i64;
                            let nanos = (timestamp.fract() * 1E9).abs() as u32;
                            FixedOffset::east_opt(0).unwrap().timestamp_opt(seconds, nanos).single()
                        }
                    }
                    _ => None
                }
            }
        }
    }

    impl Type<D1> for NaiveDateTime {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            D1TypeInfo::datetime()
        }
        fn compatible(ty: &<D1 as sqlx_core::database::Database>::TypeInfo) -> bool {
            use crate::type_info::D1Type::*;
            matches!(**ty, Datetime | Text | Integer | Real)
        }
    }
    impl Encode<'_, D1> for NaiveDateTime {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'_>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'_, D1>>::encode(self.format("%F %T%.f").to_string(), buf)
        }
    }
    impl Decode<'_, D1> for NaiveDateTime {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            Ok(<DateTime<FixedOffset> as Decode<'_, D1>>::decode(value)?.naive_local())
        }
    }

    impl Type<D1> for NaiveDate {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            D1TypeInfo::date()
        }
        fn compatible(ty: &<D1 as sqlx_core::database::Database>::TypeInfo) -> bool {
            use crate::type_info::D1Type::*;
            matches!(**ty, Date | Text)
        }
    }
    impl Encode<'_, D1> for NaiveDate {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'_>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'_, D1>>::encode(self.format("%F").to_string(), buf)
        }
    }
    impl Decode<'_, D1> for NaiveDate {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let value = value.as_string().ok_or_else(|| format!("expected `chrono::NaiveDate` but got unparsable: {value:?}"))?;
            Ok(NaiveDate::parse_from_str(&value, "%F")?)
        }
    }

    impl Type<D1> for NaiveTime {
        fn type_info() -> <D1 as sqlx_core::database::Database>::TypeInfo {
            D1TypeInfo::time()
        }
        fn compatible(ty: &<D1 as sqlx_core::database::Database>::TypeInfo) -> bool {
            use crate::type_info::D1Type::*;
            matches!(**ty, Time | Text)
        }
    }
    impl Encode<'_, D1> for NaiveTime {
        fn encode_by_ref(
            &self,
            buf: &mut <D1 as sqlx_core::database::Database>::ArgumentBuffer<'_>,
        ) -> Result<IsNull, sqlx_core::error::BoxDynError> {
            <String as Encode<'_, D1>>::encode(self.format("%T%.f").to_string(), buf)
        }
    }
    impl Decode<'_, D1> for NaiveTime {
        fn decode(value: <D1 as sqlx_core::database::Database>::ValueRef<'_>) -> Result<Self, sqlx_core::error::BoxDynError> {
            let value = value.as_string().ok_or_else(|| format!("expected `chrono::NaiveDate` but got unparsable: {value:?}"))?;
            for format in [
                "%T.f",
                "%T%.f",
                "%R",
                "%RZ",
                "%T%.fZ",
                "%R%:z",
                "%T%.f%:z",
            ] {
                if let Ok(t) = NaiveTime::parse_from_str(&value, format) {
                    return Ok(t);
                }
            }
            Err(From::from(format!("invalid time: {value:?}")))
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

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::DateTime<sqlx_core::types::chrono::Utc> | sqlx_core::types::chrono::DateTime<_>,

        #[cfg(feature = "uuid")]
        sqlx_core::types::Uuid,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
}
