mod error;
mod type_info;
mod value;
mod column;
mod row;
mod connection;
mod transaction;
mod arguments;
mod statement;
mod query_result;
mod types;

type ResultFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = Result<T, sqlx_core::Error>> + Send + 'a>>;

pub use error::D1Error;

pub use connection::{D1Connection, D1ConnectOptions};

#[derive(Debug)]
pub struct D1;

impl sqlx_core::database::Database for D1 {
    type Connection = self::connection::D1Connection;

    type TransactionManager = self::transaction::D1TransactionManager;

    type Row = self::row::D1Row;

    type QueryResult = self::query_result::D1QueryResult;

    type Column = self::column::D1Column;

    type TypeInfo = self::type_info::D1TypeInfo;

    type Value = self::value::D1Value;
    type ValueRef<'r> = self::value::D1ValueRef<'r>;

    type Arguments<'q> = self::arguments::D1Arguments;
    type ArgumentBuffer<'q> = Vec<self::value::D1Value>;

    type Statement<'q> = self::statement::D1Statement<'q>;

    const NAME: &'static str = "D1";

    const URL_SCHEMES: &'static [&'static str] = &["d1"];
}

pub mod query {
    use crate::{D1, arguments::D1Arguments, row::D1Row};
    use sqlx_core::from_row::FromRow;

    pub use sqlx_core::query::Query;
    pub fn query(sql: &str) -> Query<'_, D1, D1Arguments> {
        sqlx_core::query::query(sql)
    }
    pub fn query_with(sql: &str, args: D1Arguments) -> Query<'_, D1, D1Arguments> {
        sqlx_core::query::query_with(sql, args)
    }

    pub use sqlx_core::query_as::QueryAs;
    pub fn query_as<O>(sql: &str) -> QueryAs<'_, D1, O, D1Arguments>
    where
        O: for<'r> FromRow<'r, D1Row>,
    {
        sqlx_core::query_as::query_as(sql)
    }

    pub use sqlx_core::query_scalar::QueryScalar;
    pub fn query_scalar<S>(sql: &str) -> QueryScalar<'_, D1, S, D1Arguments>
    where
        (S,): for<'r> FromRow<'r, D1Row>,
    {
        sqlx_core::query_scalar::query_scalar(sql)
    }
}
pub use query::{query, query_as, query_scalar};

#[doc(hidden)]
pub use sqlx_core;
