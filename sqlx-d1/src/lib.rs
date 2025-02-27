pub use sqlx_d1_core::*;

#[cfg(feature = "macros")]
pub mod macros {
    #[doc(hidden)]
    pub use sqlx_d1_macros;

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
}
