mod connection;

#[derive(Debug)]
pub struct D1;

impl sqlx_core::database::Database for D1 {
    type Connection = self::connection::D1Connection;

    type TransactionManager = D1TransactionManager;

    type Row = D1Row;

    type QueryResult = D1QueryResult;

    type Column = D1Column;

    type TypeInfo = D1TypeInfo;

    type Value = D1Value;
    type ValueRef<'r> = D1ValueRef<'r>;

    type Arguments<'q> = D1Arguments<'q>;
    type ArgumentBuffer<'q> = D1ArgumentValue<'q>;

    type Statement<'q> = D1Statement<'q>;

    const NAME: &'static str = "D1";

    const URL_SCHEMES: &'static [&'static str] = &[];
}
