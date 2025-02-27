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
#[cfg(feature = "macros")]
mod macros;

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

#[doc(hidden)]
pub mod internal {
    #[cfg(feature = "macros")]
    pub use sqlx_d1_macros;
}
