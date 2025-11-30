#![cfg_attr(docsrs, feature(doc_cfg))]

mod arguments;
mod column;
mod connection;
mod error;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

type ResultFuture<'a, T> =
    std::pin::Pin<Box<dyn Future<Output = Result<T, sqlx_core::Error>> + Send + 'a>>;

pub use connection::{D1ConnectOptions, D1Connection};

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

    pub type QueryBuilder<'args> = sqlx_core::query_builder::QueryBuilder<'args, D1>;

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
    pub fn query_as_with<O>(sql: &str, args: D1Arguments) -> QueryAs<'_, D1, O, D1Arguments>
    where
        O: for<'r> FromRow<'r, D1Row>,
    {
        sqlx_core::query_as::query_as_with(sql, args)
    }

    pub use sqlx_core::query_scalar::QueryScalar;
    pub fn query_scalar<S>(sql: &str) -> QueryScalar<'_, D1, S, D1Arguments>
    where
        (S,): for<'r> FromRow<'r, D1Row>,
    {
        sqlx_core::query_scalar::query_scalar(sql)
    }
    pub fn query_scalar_with<S>(sql: &str, args: D1Arguments) -> QueryScalar<'_, D1, S, D1Arguments>
    where
        (S,): for<'r> FromRow<'r, D1Row>,
    {
        sqlx_core::query_scalar::query_scalar_with(sql, args)
    }
}
pub use query::{
    QueryBuilder, query, query_as, query_as_with, query_scalar, query_scalar_with, query_with,
};

pub use sqlx_core::Error;

#[doc(hidden)]
pub use sqlx_core;
