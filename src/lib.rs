mod error;
mod type_info;
mod column;
mod connection;
mod transaction;
mod statement;

use error::D1Error;
type ResultFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = Result<T, sqlx_core::Error>> + Send + 'a>>;

#[derive(Debug)]
pub struct D1;

impl sqlx_core::database::Database for D1 {
    type Connection = self::connection::D1Connection;

    type TransactionManager = self::transaction::D1TransactionManager;

    type Row = D1Row;

    type QueryResult = D1QueryResult;

    type Column = self::column::D1Column;

    type TypeInfo = self::type_info::D1TypeInfo;

    type Value = D1Value;
    type ValueRef<'r> = D1ValueRef<'r>;

    type Arguments<'q> = D1Arguments<'q>;
    type ArgumentBuffer<'q> = D1ArgumentValue<'q>;

    type Statement<'q> = self::statement::D1Statement<'q>;

    const NAME: &'static str = "D1";

    const URL_SCHEMES: &'static [&'static str] = &[];
}
