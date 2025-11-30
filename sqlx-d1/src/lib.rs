#![cfg_attr(docsrs, feature(doc_cfg))]
/* enable run static tests for sample codes in README */
#![cfg_attr(feature = "DEBUG", doc = include_str!("../../README.md"))]

#[cfg(feature = "query")]
#[doc(hidden)]
pub mod macros;

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use sqlx_d1_macros::FromRow;

pub use sqlx_d1_core::sqlx_core::from_row::FromRow;
pub use sqlx_d1_core::*;
