pub struct D1TransactionManager;

impl sqlx_core::transaction::TransactionManager for D1TransactionManager {
    type Database = crate::D1;

    fn begin(
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        Box::pin(async {conn.begin(); Ok(())})
    }

    fn commit(
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        conn.commit()
    }

    fn rollback(
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        Box::pin(async {conn.rollback(); Ok(())})
    }

    fn start_rollback(_: &mut <Self::Database as sqlx_core::database::Database>::Connection) {
        /* do nothing */
    }
}
