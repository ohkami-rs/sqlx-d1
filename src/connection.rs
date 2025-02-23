use sqlx_core::Url;
use std::{pin::Pin, sync::Arc};

pub struct D1Connection(
    Arc<worker::D1Database>
);
impl sqlx_core::connection::Connection for D1Connection {
    type Database = crate::D1;

    type Options = D1ConnectOptions;

    fn close(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx_core::Error>> + Send + 'static>> {
        Box::pin(async {Ok(())})
    }

    fn close_hard(self) -> Pin<Box<dyn Future<Output = Result<(), sqlx_core::Error>> + Send + 'static>> {
        Box::pin(async {Ok(())})
    }

    fn ping(&mut self) -> Pin<Box<dyn Future<Output = Result<(), sqlx_core::Error>> + Send + '_>> {
        Box::pin(async {Ok(())})
    }

    fn begin(&mut self) -> Pin<Box<dyn Future<Output = Result<sqlx_core::transaction::Transaction<'_, Self::Database>, sqlx_core::Error>> + Send + '_>>
    where
        Self: Sized,
    {
        sqlx_core::transaction::Transaction::begin(self)
    }

    fn shrink_buffers(&mut self) {
        /* do nothing */
    }

    fn flush(&mut self) -> Pin<Box<dyn Future<Output = Result<(), sqlx_core::Error>> + Send + '_>> {
        Box::pin(async {Ok(())})
    }

    fn should_flush(&self) -> bool {
        false
    }
}

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
                        .expect(todo!());
                        //.map_err(|e| sqlx_core::Error::Database(Box::new()))?;
                }
                Ok(D1Connection(Arc::new(d1)))
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
