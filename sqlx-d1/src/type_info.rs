#[derive(Debug, Clone, Copy, PartialEq)]
pub struct D1TypeInfo(D1Type);

#[derive(Debug, Clone, Copy, PartialEq)]
enum D1Type {
    Null,
    Real,
    Integer,
    Text,
    Blob,
}

const _: () = {
    impl std::fmt::Display for D1TypeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl sqlx_core::type_info::TypeInfo for D1TypeInfo {
        fn is_null(&self) -> bool {
            matches!(self.0, D1Type::Null)
        }
        
        fn name(&self) -> &str {
            match self.0 {
                D1Type::Null => "NULL",
                D1Type::Text => "TEXT",
                D1Type::Real => "REAL",
                D1Type::Blob => "BLOB",
                D1Type::Integer => "INTEGER"
            }
        }
    }

    impl D1TypeInfo {
        pub(crate) fn unknown() -> &'static Self {
            /* most least-bad choice */
            &Self(D1Type::Blob)
        }

        pub(crate) fn null() -> Self {
            Self(D1Type::Null)
        }
        pub(crate) fn real() -> Self {
            Self(D1Type::Real)
        }
        pub(crate) fn integer() -> Self {
            Self(D1Type::Integer)
        }
        pub(crate) fn text() -> Self {
            Self(D1Type::Text)
        }
        pub(crate) fn blob() -> Self {
            Self(D1Type::Blob)
        }
    }
};
