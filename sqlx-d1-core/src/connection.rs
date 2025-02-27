use sqlx_core::{Url, Either};

#[cfg(target_arch = "wasm32")]
use {
    crate::{error::D1Error, row::D1Row},
    std::pin::Pin,
    worker::{wasm_bindgen::JsValue, wasm_bindgen_futures::JsFuture, js_sys},
};

pub struct D1Connection {
    #[cfg(target_arch = "wasm32")]
    pub(crate) inner: worker_sys::D1Database,

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) inner: sqlx_sqlite::SqliteConnection,
}

const _: () = {
    /* SAFETY: used in single-threaded Workers */
    unsafe impl Send for D1Connection {}
    unsafe impl Sync for D1Connection {}

    impl D1Connection {
        #[cfg(target_arch = "wasm32")]
        pub fn new(d1: worker::D1Database) -> Self {
            Self { inner: unsafe {std::mem::transmute(d1)} }
        }

        #[cfg(not(target_arch = "wasm32"))]
        pub async fn connect(url: impl AsRef<str>) -> Result<Self, sqlx_core::Error> {
            <Self as sqlx_core::connection::Connection>::connect(url.as_ref()).await
        }
    }

    #[cfg(target_arch = "wasm32")]
    impl Clone for D1Connection {
        fn clone(&self) -> Self {
            Self { inner: self.inner.clone() }
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
            #[allow(unused)]
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
            #[cfg(not(target_arch = "wasm32"))] {
                unreachable!("native `Executor` impl")
            }
            #[cfg(target_arch = "wasm32")] {
                <&'c D1Connection as sqlx_core::executor::Executor<'c>>::fetch_many(self, query)
            }
        }

        fn fetch_optional<'e, 'q: 'e, E>(
            self,
            #[allow(unused)]
            mut query: E,
        ) -> crate::ResultFuture<'e, Option<<Self::Database as sqlx_core::database::Database>::Row>>
        where
            'c: 'e,
            E: 'q + sqlx_core::executor::Execute<'q, Self::Database>,
        {
            #[cfg(not(target_arch = "wasm32"))] {
                unreachable!("native `Executor` impl")
            }
            #[cfg(target_arch = "wasm32")] {
                <&'c D1Connection as sqlx_core::executor::Executor<'c>>::fetch_optional(self, query)
            }
        }

        fn prepare_with<'e, 'q: 'e>(
            self,
            sql: &'q str,
            _parameters: &'e [<Self::Database as sqlx_core::database::Database>::TypeInfo],
        ) -> crate::ResultFuture<'e, <Self::Database as sqlx_core::database::Database>::Statement<'q>>
        where
            'c: 'e,
        {
            Box::pin(async {
                Ok(crate::statement::D1Statement {
                    sql: std::borrow::Cow::Borrowed(sql),
                })
            })
        }

        fn describe<'e, 'q: 'e>(
            self,
            #[allow(unused)]
            sql: &'q str,
        ) -> crate::ResultFuture<'e, sqlx_core::describe::Describe<Self::Database>>
        where
            'c: 'e,
        {
            #[cfg(target_arch = "wasm32")] {
                unreachable!("wasm32 describe")
            }
            #[cfg(not(target_arch = "wasm32"))] {
                /* compile-time verification by macros */

                Box::pin(async {
                    let sqlx_core::describe::Describe {
                        columns,
                        parameters,
                        nullable
                    } = <&mut sqlx_sqlite::SqliteConnection as sqlx_core::executor::Executor>::describe(
                        &mut self.inner,
                        sql
                    ).await?;
                    
                    Ok(sqlx_core::describe::Describe {
                        parameters: parameters.map(|ps| match ps {
                            Either::Left(type_infos) => Either::Left(type_infos.into_iter().map(crate::type_info::D1TypeInfo::from_sqlite).collect()),
                            Either::Right(n) => Either::Right(n)
                        }),
                        columns: columns.into_iter().map(crate::column::D1Column::from_sqlite).collect(),
                        nullable
                    })
                })
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    impl<'c> sqlx_core::executor::Executor<'c> for &'c D1Connection {
        type Database = crate::D1;

        fn fetch_many<'e, 'q: 'e, E>(
            self,
            #[allow(unused)]
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
                raw_rows: Option<js_sys::ArrayIntoIter>,
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
                    F: Future<Output = Result<Option<js_sys::Array>, JsValue>>,
                {
                    type Item = Result<
                        Either<crate::query_result::D1QueryResult, D1Row>,
                        sqlx_core::Error
                    >;

                    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
                        use std::task::Poll;

                        fn pop_next(raw_rows: &mut js_sys::ArrayIntoIter) ->
                            Option<Result<
                                Either<crate::query_result::D1QueryResult, D1Row>,
                                sqlx_core::Error
                            >>
                        {
                            let raw_row = raw_rows.next()?;
                            Some(D1Row::from_raw(raw_row).map(Either::Right))
                        }

                        let this = unsafe {self.get_unchecked_mut()};
                        match &mut this.raw_rows {
                            Some(raw_rows) => Poll::Ready(pop_next(raw_rows)),
                            None => match unsafe {Pin::new_unchecked(&mut this.raw_rows_future)}.poll(cx) {
                                Poll::Pending => Poll::Pending,
                                Poll::Ready(Err(e)) => Poll::Ready(Some(Err(
                                    sqlx_core::Error::from(D1Error::from(e))
                                ))),
                                Poll::Ready(Ok(maybe_raw_rows)) => {
                                    this.raw_rows = Some(maybe_raw_rows.unwrap_or_else(js_sys::Array::new).into_iter());
                                    Poll::Ready(pop_next(unsafe {this.raw_rows.as_mut().unwrap_unchecked()}))
                                }
                            }
                        }                        
                    }
                }
            };

            Box::pin(FetchMany::new(async move {
                let mut statement = self.inner.prepare(sql).unwrap();
                if let Some(a) = arguments {
                    statement = statement.bind(a.as_ref().iter().collect())?;
                }

                let d1_result_jsvalue = JsFuture::from(statement.all()?)
                    .await?;
                worker_sys::D1Result::from(d1_result_jsvalue)
                    .results()
            }))
        }

        fn fetch_optional<'e, 'q: 'e, E>(
            self,
            #[allow(unused)]
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
                let mut statement = self.inner.prepare(sql).unwrap();
                if let Some(a) = arguments {
                    statement = statement
                        .bind(a.as_ref().iter().collect())
                        .map_err(|e| sqlx_core::Error::Encode(Box::new(D1Error::from(e))))?;
                }

                let raw = JsFuture::from(statement.first(None).map_err(D1Error::from)?)
                    .await
                    .map_err(D1Error::from)?;
                if raw.is_null() {
                    Ok(None)
                } else {
                    D1Row::from_raw(raw).map(Some)
                }
            }))
        }

        fn prepare_with<'e, 'q: 'e>(
            self,
            sql: &'q str,
            _parameters: &'e [<Self::Database as sqlx_core::database::Database>::TypeInfo],
        ) -> crate::ResultFuture<'e, <Self::Database as sqlx_core::database::Database>::Statement<'q>>
        where
            'c: 'e,
        {
            Box::pin(async {
                Ok(crate::statement::D1Statement {
                    sql: std::borrow::Cow::Borrowed(sql),
                })
            })
        }

        fn describe<'e, 'q: 'e>(
            self,
            #[allow(unused)]
            sql: &'q str,
        ) -> crate::ResultFuture<'e, sqlx_core::describe::Describe<Self::Database>>
        where
            'c: 'e,
        {
            unreachable!("wasm32 describe")
        }
    }
};

/// ref: <https://developers.cloudflare.com/d1/sql-api/sql-statements/#compatible-pragma-statements>
#[derive(Clone)]
pub struct D1ConnectOptions {
    pragmas: TogglePragmas,
    #[cfg(target_arch = "wasm32")]
    d1: worker_sys::D1Database,
    #[cfg(not(target_arch = "wasm32"))]
    sqlite_path: std::path::PathBuf,
}
const _: () = {
    /* SAFETY: used in single-threaded Workers */
    unsafe impl Send for D1ConnectOptions {}
    unsafe impl Sync for D1ConnectOptions {}

    #[cfg(target_arch = "wasm32")]
    const URL_CONVERSION_UNSUPPORTED_MESSAGE: &'static str = "\
        `sqlx_d1::D1ConnectOptions` doesn't support conversion between `Url`. \
        Consider connect from options created by `D1ConnectOptions::new`. \
    ";

    const LOG_SETTINGS_UNSUPPORTED_MESSAGE: &'static str = "\
        `sqlx_d1::D1ConnectOptions` doesn't support log settings.
    ";

    impl D1ConnectOptions {
        #[cfg(target_arch = "wasm32")]
        pub fn new(d1: worker::D1Database) -> Self {
            Self {
                d1: unsafe {core::mem::transmute(d1)},
                pragmas: TogglePragmas::new(),
            }
        }
    }

    impl std::fmt::Debug for D1ConnectOptions {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D1ConnectOptions")
                .field("pragmas", &self.pragmas)
                .finish()
        }
    }

    impl std::str::FromStr for D1ConnectOptions {
        type Err = sqlx_core::Error;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            #[cfg(target_arch = "wasm32")] {
                Err(sqlx_core::Error::Configuration(From::from(
                    URL_CONVERSION_UNSUPPORTED_MESSAGE
                )))
            }

            #[cfg(not(target_arch = "wasm32"))] {
                use std::{io, fs, path::{Path, PathBuf}};

                fn maybe_miniflare_d1_dir_of(dir: impl AsRef<Path>) -> PathBuf {
                    dir.as_ref()
                        .join(".wrangler")
                        .join("state")
                        .join("v3")
                        .join("d1")
                        .join("miniflare-D1DatabaseObject")
                }
            
                const PACKAGE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

                let (candidate_1, candidate_2) = (
                    maybe_miniflare_d1_dir_of(PACKAGE_ROOT),
                    maybe_miniflare_d1_dir_of(".")
                );

                let sqlite_path = (|| -> io::Result<PathBuf> {                    
                    let miniflare_d1_dir = match (
                        fs::exists(&candidate_1),
                        fs::exists(&candidate_2)
                    ) {
                        (Ok(true), _) => candidate_1,
                        (_, Ok(true)) => candidate_2,
                        (Err(e), _) | (_, Err(e)) => return Err(e),
                        (Ok(false), Ok(false)) => return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "miniflare's D1 emulating directory not found"
                        )),
                    };
                    
                    let [sqlite_path] = fs::read_dir(miniflare_d1_dir)?
                        .filter_map(|r| r.as_ref().ok().and_then(|e| {
                            let path = e.path();
                            path.extension()
                                .is_some_and(|ex| ex == "sqlite")
                                .then_some(path)
                        }))
                        .collect::<Vec<_>>()
                        .try_into()
                        .map_err(|_| io::Error::new(
                            io::ErrorKind::Other,
                            "Currently, sqlx_d1 doesn't support multiple D1 bindings!"
                        ))?;

                    Ok(sqlite_path)
                })().map_err(|_| sqlx_core::Error::WorkerCrashed)?;
                    
                Ok(Self {
                    pragmas: TogglePragmas::new(),
                    sqlite_path
                })
            }
        }
    }

    impl sqlx_core::connection::ConnectOptions for D1ConnectOptions {
        type Connection = D1Connection;

        fn from_url(_url: &Url) -> Result<Self, sqlx_core::Error> {
            #[cfg(target_arch = "wasm32")] {
                Err(sqlx_core::Error::Configuration(From::from(
                    URL_CONVERSION_UNSUPPORTED_MESSAGE
                )))
            }
            #[cfg(not(target_arch = "wasm32"))] {
                _url.as_str().parse()
            }
        }

        fn to_url_lossy(&self) -> Url {
            unreachable!("`sqlx_d1::ConnectOptions` doesn't support `ConnectOptions::to_url_lossy`")
        }

        fn connect(&self) -> crate::ResultFuture<'_, Self::Connection>
        where
            Self::Connection: Sized,
        {
            #[cfg(target_arch = "wasm32")] {
                Box::pin(worker::send::SendFuture::new(async move {
                    let d1 = self.d1.clone();

                    if let Some(pragmas) = self.pragmas.collect() {
                        JsFuture::from(d1.exec(&pragmas.join("\n")).map_err(D1Error::from)?)
                            .await
                            .map_err(D1Error::from)?;
                    }

                    Ok(D1Connection {
                        inner: d1
                    })
                }))
            }

            #[cfg(not(target_arch = "wasm32"))] {
                Box::pin(async move {
                    use sqlx_core::{connection::Connection, executor::Executor};

                    let mut sqlite_conn = sqlx_sqlite::SqliteConnection::connect(
                        self.sqlite_path.to_str().ok_or(sqlx_core::Error::WorkerCrashed)?
                    ).await?;

                    if let Some(pragmas) = self.pragmas.collect() {
                        for pragma in pragmas {
                            sqlite_conn.execute(pragma).await?;
                        }
                    }
                    
                    Ok(D1Connection { inner: sqlite_conn })
                })
            }
        }

        fn log_statements(self, _: log::LevelFilter) -> Self {
            unreachable!("{LOG_SETTINGS_UNSUPPORTED_MESSAGE}")
        }

        fn log_slow_statements(self, _: log::LevelFilter, _: std::time::Duration) -> Self {
            unreachable!("{LOG_SETTINGS_UNSUPPORTED_MESSAGE}")
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
