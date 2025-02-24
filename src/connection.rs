use crate::row::D1Row;
use sqlx_core::{Url, Either};
use worker::wasm_bindgen::JsValue;
use std::{pin::Pin, sync::Arc};

pub struct D1Connection {
    pub(crate) inner: Arc<worker::D1Database>,
    pub(crate) batch: Option<worker::send::SendWrapper<Box<Vec<worker::D1PreparedStatement>>>>,
}
const _: () = {
    impl D1Connection {
        pub(crate) fn begin(&mut self) {
            self.batch = Some(worker::send::SendWrapper(Box::new(Vec::new())));
        }

        pub(crate) fn commit(&mut self) -> crate::ResultFuture<'_, ()> {
            Box::pin(worker::send::SendFuture::new(async {
                if let Some(batch) = &mut self.batch {
                    self.inner.batch(std::mem::take(&mut batch.0)).await
                        .map_err(|e| sqlx_core::Error::Database(Box::new(crate::D1Error::from(e))))?;
                }
                Ok(())
            }))
        }

        pub(crate) fn rollback(&mut self) {
            self.batch = None;
        }
    }

    impl std::fmt::Debug for D1Connection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D1Connection").finish()
        }
    }

    impl sqlx_core::connection::Connection for D1Connection {
        type Database = crate::D1;

        type Options = D1ConnectOptions;

        fn close(self) -> crate::ResultFuture<'static, ()> {
            Box::pin(async {Ok(())})
        }

        fn close_hard(self) -> crate::ResultFuture<'static, ()> {
            Box::pin(async {Ok(())})
        }

        fn ping(&mut self) -> crate::ResultFuture<'_, ()> {
            Box::pin(async {Ok(())})
        }

        fn begin(&mut self) -> crate::ResultFuture<'_, sqlx_core::transaction::Transaction<'_, Self::Database>>
        where
            Self: Sized,
        {
            sqlx_core::transaction::Transaction::begin(self)
        }

        fn shrink_buffers(&mut self) {
            /* do nothing */
        }

        fn flush(&mut self) -> crate::ResultFuture<'_, ()> {
            Box::pin(async {Ok(())})
        }

        fn should_flush(&self) -> bool {
            false
        }
    }

    impl<'c> sqlx_core::executor::Executor<'c> for &'c mut D1Connection {
        type Database = crate::D1;

        fn fetch_many<'e, 'q: 'e, E>(
            self,
            mut query: E,
        ) -> futures_core::stream::BoxStream<
            'e,
            Result<
                Either<
                    <Self::Database as sqlx_core::database::Database>::QueryResult,
                    <Self::Database as sqlx_core::database::Database>::Row
                >,
                sqlx_core::Error,
            >,
        >
        where
            'c: 'e,
            E: 'q + sqlx_core::executor::Execute<'q, Self::Database>,
        {
            let sql = query.sql();
            let arguments = match query.take_arguments() {
                Ok(a) => a,
                Err(e) => return Box::pin(futures_util::stream::once(async {Err(sqlx_core::Error::Encode(e))})),
            };

            struct FetchMany<F> {
                raw_rows_future: F,
                raw_rows: Option<Vec<JsValue>>,
            }
            const _: () = {
                /* SAFETY: used in single-threaded Workers */
                unsafe impl<F> Send for FetchMany<F> {}

                impl<F> FetchMany<F> {
                    fn new(raw_rows_future: F) -> Self {
                        Self { raw_rows_future, raw_rows: None }
                    }
                }

                impl<F> futures_core::Stream for FetchMany<F>
                where
                    F: Future<Output = Result<Vec<JsValue>, worker::Error>>,
                {
                    type Item = Result<
                        Either<crate::query_result::D1QueryResult, D1Row>,
                        sqlx_core::Error
                    >;

                    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
                        use std::task::Poll;

                        fn pop_next(raw_rows: &mut Vec<JsValue>) ->
                            Option<Result<
                                Either<crate::query_result::D1QueryResult, D1Row>,
                                sqlx_core::Error
                            >>
                        {
                            let raw_row = raw_rows.pop()?;
                            Some(D1Row::from_raw(raw_row).map(Either::Right))
                        }

                        let this = unsafe {self.get_unchecked_mut()};
                        match &mut this.raw_rows {
                            Some(raw_rows) => Poll::Ready(pop_next(raw_rows)),
                            None => match unsafe {Pin::new_unchecked(&mut this.raw_rows_future)}.poll(cx) {
                                Poll::Pending => Poll::Pending,
                                Poll::Ready(Err(e)) => Poll::Ready(Some(Err(
                                    sqlx_core::Error::Database(Box::new(crate::D1Error::from_rust(e)))
                                ))),
                                Poll::Ready(Ok(raw_rows)) => {
                                    this.raw_rows = Some(raw_rows);
                                    Poll::Ready(pop_next(unsafe {this.raw_rows.as_mut().unwrap_unchecked()}))
                                }
                            }
                        }                        
                    }
                }
            };

            Box::pin(FetchMany::new(async move {
                let mut statement = self.inner.prepare(sql);
                if let Some(a) = arguments {
                    statement = statement.bind(a.as_ref())?;
                }
                statement.raw_js_value().await
            }))
        }

        fn fetch_optional<'e, 'q: 'e, E>(
            self,
            mut query: E,
        ) -> crate::ResultFuture<'e, Option<<Self::Database as sqlx_core::database::Database>::Row>>
        where
            'c: 'e,
            E: 'q + sqlx_core::executor::Execute<'q, Self::Database>,
        {
            let sql = query.sql();
            let arguments = match query.take_arguments() {
                Ok(a) => a,
                Err(e) => return Box::pin(async {Err(sqlx_core::Error::Encode(e))}),
            };

            Box::pin(worker::send::SendFuture::new(async move {
                let mut statement = self.inner.prepare(sql);
                if let Some(a) = arguments {
                    statement = statement.bind(a.as_ref())
                        .map_err(|e| sqlx_core::Error::Encode(Box::new(crate::D1Error::from_rust(e))))?;
                }

                let mut raw_rows = statement.raw_js_value().await
                    .map_err(crate::D1Error::from_rust)?;

                raw_rows.pop().map(D1Row::from_raw).transpose()
            }))
        }

        fn prepare_with<'e, 'q: 'e>(
            self,
            sql: &'q str,
            parameters: &'e [<Self::Database as sqlx_core::database::Database>::TypeInfo],
        ) -> futures_core::future::BoxFuture<'e, Result<<Self::Database as sqlx_core::database::Database>::Statement<'q>, sqlx_core::Error>>
        where
            'c: 'e,
        {
            todo!()
        }

        fn describe<'e, 'q: 'e>(
            self,
            sql: &'q str,
        ) -> crate::ResultFuture<'e, sqlx_core::describe::Describe<Self::Database>>
        where
            'c: 'e,
        {
            todo!()
        }
    }
};

/// ref: <https://developers.cloudflare.com/d1/sql-api/sql-statements/#compatible-pragma-statements>
#[derive(Clone)]
pub struct D1ConnectOptions {
    env: worker::Env,
    binding: &'static str,
    pragmas: TogglePragmas,
}
const _: () = {
    const URL_CONVERSION_UNSUPPORTED_MESSAGE: &'static str = "\
        `sqlx_d1::D1ConnectOptions` doesn't support conversion between `Url`. \
        Consider connect from options created by `D1ConnectOptions::new`. \
    ";

    const LOG_SETTINGS_UNSUPPORTED_MESSAGE: &'static str = "\
        `sqlx_d1::D1ConnectOptions` doesn't support log settings.
    ";

    impl D1ConnectOptions {
        pub fn new(env: worker::Env, binding: &'static str) -> Self {
            Self { env, binding, pragmas: TogglePragmas::new() }
        }
    }

    impl std::fmt::Debug for D1ConnectOptions {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D1ConnectOptions")
                .field("binding", &self.binding)
                .field("pragmas", &self.pragmas)
                .finish()
        }
    }

    impl std::str::FromStr for D1ConnectOptions {
        type Err = sqlx_core::Error;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Err(sqlx_core::Error::Configuration(From::from(
                URL_CONVERSION_UNSUPPORTED_MESSAGE
            )))
        }
    }

    impl sqlx_core::connection::ConnectOptions for D1ConnectOptions {
        type Connection = D1Connection;

        fn from_url(_: &Url) -> Result<Self, sqlx_core::Error> {
            Err(sqlx_core::Error::Configuration(From::from(
                URL_CONVERSION_UNSUPPORTED_MESSAGE
            )))
        }

        fn to_url_lossy(&self) -> Url {
            unreachable!("{}", URL_CONVERSION_UNSUPPORTED_MESSAGE)
        }

        fn connect(&self) -> Pin<Box<dyn Future<Output = Result<Self::Connection, sqlx_core::Error>> + Send + '_>>
        where
            Self::Connection: Sized,
        {
            Box::pin(worker::send::SendFuture::new(async move {
                let d1 = self.env.d1(self.binding)
                    .map_err(|e| sqlx_core::Error::Configuration(Box::new(e)))?;
                if let Some(pragmas) = self.pragmas.collect() {
                    d1.exec(&pragmas.join("\n")).await
                        .map_err(|e| sqlx_core::Error::Database(Box::new(crate::D1Error::from(e))))?;
                }
                Ok(D1Connection {
                    inner: Arc::new(d1),
                    batch: None,
                })
            }))
        }

        fn log_statements(self, _: log::LevelFilter) -> Self {
            unreachable!("{}", LOG_SETTINGS_UNSUPPORTED_MESSAGE)
        }

        fn log_slow_statements(self, _: log::LevelFilter, _: std::time::Duration) -> Self {
            unreachable!("{}", LOG_SETTINGS_UNSUPPORTED_MESSAGE)
        }
    }
};

/// ref: <https://developers.cloudflare.com/d1/sql-api/sql-statements/#compatible-pragma-statements>
#[derive(Clone, Copy)]
struct TogglePragmas(u8);
const _: () = {
    impl std::ops::Not for TogglePragmas {
        type Output = Self;
        fn not(self) -> Self::Output {
            Self(!self.0)
        }
    }
    impl std::ops::BitOrAssign for TogglePragmas {
        fn bitor_assign(&mut self, rhs: Self) {
            self.0 |= self.0 | rhs.0;
        }
    }
    impl std::ops::BitAndAssign for TogglePragmas {
        fn bitand_assign(&mut self, rhs: Self) {
            self.0 &= self.0 & rhs.0;
        }
    }
    
    impl TogglePragmas {
        const fn new() -> Self {
            Self(0)
        }
    }
};

macro_rules! toggles {
    ($( $name:ident as $bits:literal; )*) => {
        impl TogglePragmas {
            $(
                #[allow(non_upper_case_globals)]
                const $name: Self = Self($bits);
            )*

            fn collect(&self) -> Option<Vec<&'static str>> {
                #[allow(unused_mut)]
                let mut pragmas = Vec::new();
                $(
                    if self.0 & Self::$name.0 != 0 {
                        pragmas.push(concat!(
                            "PRAGMA ",
                            stringify!($name),
                            " = on"
                        ));
                    }
                )*
                (!pragmas.is_empty()).then_some(pragmas)
            }
        }

        impl std::fmt::Debug for TogglePragmas {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let mut f = &mut f.debug_map();
                $(
                    f = f.entry(
                        &stringify!($name),
                        &if self.0 & Self::$name.0 != 0 {"on"} else {"off"}
                    );
                )*
                f.finish()
            }
        }

        impl D1ConnectOptions {
            $(
                pub fn $name(mut self, yes: bool) -> Self {
                    if yes {
                        self.pragmas |= TogglePragmas::$name;
                    } else {
                        self.pragmas &= !TogglePragmas::$name;
                    }
                    self
                }
            )*
        }
    };
}
toggles! {
    case_sensitive_like     as 0b0000001;
    ignore_check_constraint as 0b0000010;
    legacy_alter_table      as 0b0000100;
    recursive_triggers      as 0b0001000;
    unordered_selects       as 0b0010000;
    foreign_keys            as 0b0100000;
    defer_foreign_keys      as 0b1000000;
}
