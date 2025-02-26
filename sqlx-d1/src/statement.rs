use std::borrow::Cow;
use sqlx_core::impl_statement_query;

pub struct D1Statement<'q> {
    pub(crate) sql: Cow<'q, str>,
}

impl<'q> sqlx_core::statement::Statement<'q> for D1Statement<'q> {
    type Database = crate::D1;

    fn to_owned(&self) -> <Self::Database as sqlx_core::database::Database>::Statement<'static> {
        D1Statement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<sqlx_core::Either<&[<Self::Database as sqlx_core::database::Database>::TypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[<Self::Database as sqlx_core::database::Database>::Column] {
        &[]
    }

    impl_statement_query!(crate::arguments::D1Arguments);
}
