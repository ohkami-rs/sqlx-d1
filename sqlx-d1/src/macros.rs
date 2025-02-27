#![cfg(feature = "macros")]

//! ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/src/macros/mod.rs>

#[doc(hidden)]
pub use sqlx_d1_macros;

/// `sqlx::query!` for Cloudflare D1.
/// 
/// See [sqlx::query!](https://docs.rs/sqlx/latest/sqlx/macro.query.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query (
    ($query:expr) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source = $query)
    });
    ($query:expr, $($args:expr),* $(,)?) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source = $query, args = [$($args)*])
    })
);

/// `sqlx::query_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_unchecked (
    ($query:expr) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source = $query, checked = false)
    });
    ($query:expr, $($args:expr),* $(,)?) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source = $query, args = [$($args)*], checked = false)
    })
);

/// `sqlx::query_file!` for Cloudflare D1.
/// 
/// See [sqlx::query_file!](https://docs.rs/sqlx/latest/sqlx/macro.query_file.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file (
    ($path:literal) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source_file = $path)
    });
    ($path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source_file = $path, args = [$($args)*])
    })
);

/// `sqlx::query_file_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_file_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_unchecked (
    ($path:literal) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source_file = $path, checked = false)
    });
    ($path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_d1_macros::expand_query!(source_file = $path, args = [$($args)*], checked = false)
    })
);

/// `sqlx::query_as!` for Cloudflare D1.
/// 
/// See [sqlx::query_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as (
    ($out_struct:path, $query:expr) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query)
    });
    ($out_struct:path, $query:expr, $($args:tt)*) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*])
    })
);

/// `sqlx::query_file_as!` for Cloudflare D1.
/// 
/// See [sqlx::query_file_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_as.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_as (
    ($out_struct:path, $path:literal) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path)
    });
    ($out_struct:path, $path:literal, $($args:tt)*) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args)*])
    })
);

/// `sqlx::query_as_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_as_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_as_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as_unchecked (
    ($out_struct:path, $query:expr) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, checked = false)
    });

    ($out_struct:path, $query:expr, $($args:tt)*) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*], checked = false)
    })
);

/// `sqlx::query_file_as_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_file_as_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_as_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_as_unchecked (
    ($out_struct:path, $path:literal) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path, checked = false)
    });

    ($out_struct:path, $path:literal, $($args:tt)*) => ( {
        $crate::macros::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args)*], checked = false)
    })
);

/// `sqlx::query_scalor!` for Cloudflare D1.
/// 
/// See [sqlx::query_scalor!](https://docs.rs/sqlx/latest/sqlx/macro.query_scalor.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_scalar (
    ($query:expr) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source = $query)
    );
    ($query:expr, $($args:tt)*) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source = $query, args = [$($args)*])
    )
);

/// `sqlx::query_file_scalor!` for Cloudflare D1.
/// 
/// See [sqlx::query_file_scalor!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_scalor.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_scalar (
    ($path:literal) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path)
    );
    ($path:literal, $($args:tt)*) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*])
    )
);

/// `sqlx::query_scalor_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_scalor_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_scalor_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_scalar_unchecked (
    ($query:expr) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source = $query, checked = false)
    );
    ($query:expr, $($args:tt)*) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source = $query, args = [$($args)*], checked = false)
    )
);

/// `sqlx::query_file_scalor_unchecked!` for Cloudflare D1.
/// 
/// See [sqlx::query_file_scalor_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_scalor_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_scalar_unchecked (
    ($path:literal) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, checked = false)
    );
    ($path:literal, $($args:tt)*) => (
        $crate::macros::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*], checked = false)
    )
);
