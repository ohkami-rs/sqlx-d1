pub struct D1TransactionManager;

impl sqlx_core::transaction::TransactionManager for D1TransactionManager {
    type Database = crate::D1;

    fn begin(
        #[allow(unused)]
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        #[cfg(not(target_arch = "wasm32"))] {
            unreachable!("Native `TransactionManager` impl")
        }
        #[cfg(target_arch = "wasm32")] {
            Box::pin(async {conn.begin(); Ok(())})
        }
    }

    fn commit(
        #[allow(unused)]
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        #[cfg(not(target_arch = "wasm32"))] {
            unreachable!("Native `TransactionManager` impl")
        }
        #[cfg(target_arch = "wasm32")] {
            conn.commit()
        }
    }

    fn rollback(
        #[allow(unused)]
        conn: &mut <Self::Database as sqlx_core::database::Database>::Connection,
    ) -> crate::ResultFuture<'_, ()> {
        #[cfg(not(target_arch = "wasm32"))] {
            unreachable!("Native `TransactionManager` impl")
        }
        #[cfg(target_arch = "wasm32")] {
            Box::pin(async {conn.rollback(); Ok(())})
        }
    }

    fn start_rollback(_: &mut <Self::Database as sqlx_core::database::Database>::Connection) {
        /* do nothing */
    }
}
