#![cfg(feature = "macros")]

/// ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/src/macros/mod.rs>

#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query {
    ($query:expr) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source = $query)
    });
    ($query:expr, $($args:expr),* $(,)?) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source = $query, args = [$($args)*])
    })
}

/// A variant of [`query!`][`crate::query!`] which does not check the input or output types. This still does parse
/// the query to ensure it's syntactically and semantically valid for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_unchecked (
    ($query:expr) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source = $query, checked = false)
    });
    ($query:expr, $($args:expr),* $(,)?) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source = $query, args = [$($args)*], checked = false)
    })
);

/// A variant of [`query!`][`crate::query!`] where the SQL query is stored in a separate file.
///
/// Useful for large queries and potentially cleaner than multiline strings.
///
/// The syntax and requirements (see [`query!`][`crate::query!`]) are the same except the SQL
/// string is replaced by a file path.
///
/// The file must be relative to the project root (the directory containing `Cargo.toml`),
/// unlike `include_str!()` which uses compiler internals to get the path of the file where it
/// was invoked.
///
/// -----
///
/// `examples/queries/account-by-id.sql`:
/// ```text
/// select * from (select (1) as id, 'Herp Derpinson' as name) accounts
/// where id = ?
/// ```
///
/// `src/my_query.rs`:
/// ```rust,ignore
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "_rt-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// let account = sqlx::query_file!("tests/test-query-account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{account:?}");
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "_rt-async-std")))]
/// # fn main() {}
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file (
    ($path:literal) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source_file = $path)
    });
    ($path:literal, $($args:expr),* $(,)?) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source_file = $path, args = [$($args)*])
    })
);

/// A variant of [`query_file!`][`crate::query_file!`] which does not check the input or output
/// types. This still does parse the query to ensure it's syntactically and semantically valid
/// for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_unchecked (
    ($path:literal) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source_file = $path, checked = false)
    });
    ($path:literal, $($args:expr),* $(,)?) => ({
        $crate::internal::sqlx_d1_macros::expand_query!(source_file = $path, args = [$($args)*], checked = false)
    })
);

/// A variant of [`query!`][`crate::query!`] which takes a path to an explicitly defined struct
/// as the output type.
///
/// This lets you return the struct from a function or add your own trait implementations.
///
/// **This macro does not use [`FromRow`][crate::FromRow]**; in fact, no trait implementations are
/// required at all, though this may change in future versions.
///
/// The macro maps rows using a struct literal where the names of columns in the query are expected
/// to be the same as the fields of the struct (but the order does not need to be the same).
/// The types of the columns are based on the query and not the corresponding fields of the struct,
/// so this is type-safe as well.
///
/// This enforces a few things:
/// * The query must output at least one column.
/// * The column names of the query must match the field names of the struct.
/// * The field types must be the Rust equivalent of SQL counterparts; see [`crate::types`]
/// * If a column may be `NULL`, the corresponding field's type must be wrapped in `Option<_>`.
/// * Neither the query nor the struct may have unused fields.
///
/// The only modification to the `query!()` syntax is that the struct name is given before the SQL
/// string:
/// ```rust,ignore
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "_rt-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query_as!(
///         Account,
///         "select * from (select (1) as id, 'Herp Derpinson' as name) accounts where id = ?",
///         1i32
///     )
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{account:?}");
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "_rt-async-std")))]
/// # fn main() {}
/// ```
///
/// **The method you want to call depends on how many rows you're expecting.**
///
/// | Number of Rows | Method to Call*             | Returns (`T` being the given struct)   | Notes |
/// |----------------| ----------------------------|----------------------------------------|-------|
/// | Zero or One    | `.fetch_optional(...).await`| `sqlx::Result<Option<T>>`              | Extra rows are ignored. |
/// | Exactly One    | `.fetch_one(...).await`     | `sqlx::Result<T>`                      | Errors if no rows were returned. Extra rows are ignored. Aggregate queries, use this. |
/// | At Least One   | `.fetch(...)`               | `impl Stream<Item = sqlx::Result<T>>`  | Call `.try_next().await` to get each row result. |
/// | Multiple       | `.fetch_all(...)`           | `sqlx::Result<Vec<T>>`  | |
///
/// \* All methods accept one of `&mut {connection type}`, `&mut Transaction` or `&Pool`.
/// (`.execute()` is omitted as this macro requires at least one column to be returned.)
///
/// ### Column Type Override: Infer from Struct Field
/// In addition to the column type overrides supported by [`query!`][`crate::query!`],
/// [`query_as!()`][`crate::query_as!`] supports an
/// additional override option:
///
/// If you select a column `foo as "foo: _"`
/// it causes that column to be inferred based on the type of the corresponding field in the given
/// record struct. Runtime type-checking is still done so an error will be emitted if the types
/// are not compatible.
///
/// This allows you to override the inferred type of a column to instead use a custom-defined type:
///
/// ```rust,ignore
/// #[derive(sqlx::Type)]
/// #[sqlx(transparent)]
/// struct MyInt4(i32);
///
/// struct Record {
///     id: MyInt4,
/// }
///
/// let my_int = MyInt4(1);
///
/// // Postgres/SQLite
/// sqlx::query_as!(Record, r#"select 1 as "id: _""#) // MySQL: use "select 1 as `id: _`" instead
///     .fetch_one(&mut conn)
///     .await?;
///
/// assert_eq!(record.id, MyInt4(1));
/// ```
///
/// ### Troubleshooting: "error: mismatched types"
/// If you get a "mismatched types" error from an invocation of this macro and the error
/// isn't pointing specifically at a parameter.
///
/// For example, code like this (using a Postgres database):
///
/// ```rust,ignore
/// struct Account {
///     id: i32,
///     name: Option<String>,
/// }
///
/// let account = sqlx::query_as!(
///     Account,
///     r#"SELECT id, name from (VALUES (1, 'Herp Derpinson')) accounts(id, name)"#,
/// )
///     .fetch_one(&mut conn)
///     .await?;
/// ```
///
/// Might produce an error like this:
/// ```text,ignore
/// error[E0308]: mismatched types
///    --> tests/postgres/macros.rs:126:19
///     |
/// 126 |       let account = sqlx::query_as!(
///     |  ___________________^
/// 127 | |         Account,
/// 128 | |         r#"SELECT id, name from (VALUES (1, 'Herp Derpinson')) accounts(id, name)"#,
/// 129 | |     )
///     | |_____^ expected `i32`, found enum `std::option::Option`
///     |
///     = note: expected type `i32`
///                found enum `std::option::Option<i32>`
/// ```
///
/// This means that you need to check that any field of the "expected" type (here, `i32`) matches
/// the Rust type mapping for its corresponding SQL column (see the `types` module of your database,
/// listed above, for mappings). The "found" type is the SQL->Rust mapping that the macro chose.
///
/// In the above example, the returned column is inferred to be nullable because it's being
/// returned from a `VALUES` statement in Postgres, so the macro inferred the field to be nullable
/// and so used `Option<i32>` instead of `i32`. **In this specific case** we could use
/// `select id as "id!"` to override the inferred nullability because we know in practice
/// that column will never be `NULL` and it will fix the error.
///
/// Nullability inference and type overrides are discussed in detail in the docs for
/// [`query!`][`crate::query!`].
///
/// It unfortunately doesn't appear to be possible right now to make the error specifically mention
/// the field; this probably requires the `const-panic` feature (still unstable as of Rust 1.45).
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as (
    ($out_struct:path, $query:expr) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query)
    });
    ($out_struct:path, $query:expr, $($args:expr),* $(,)?) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*])
    })
);

/// Combines the syntaxes of [`query_as!`][`crate::query_as!`] and [`query_file!`][`crate::query_file!`].
///
/// Enforces requirements of both macros; see them for details.
///
/// ```rust,ignore
/// # use sqlx::Connect;
/// # #[cfg(all(feature = "mysql", feature = "_rt-async-std"))]
/// # #[async_std::main]
/// # async fn main() -> sqlx::Result<()>{
/// # let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// #
/// # if !(db_url.starts_with("mysql") || db_url.starts_with("mariadb")) { return Ok(()) }
/// # let mut conn = sqlx::MySqlConnection::connect(db_url).await?;
/// #[derive(Debug)]
/// struct Account {
///     id: i32,
///     name: String
/// }
///
/// // let mut conn = <impl sqlx::Executor>;
/// let account = sqlx::query_file_as!(Account, "tests/test-query-account-by-id.sql", 1i32)
///     .fetch_one(&mut conn)
///     .await?;
///
/// println!("{account:?}");
/// println!("{}: {}", account.id, account.name);
///
/// # Ok(())
/// # }
/// #
/// # #[cfg(any(not(feature = "mysql"), not(feature = "_rt-async-std")))]
/// # fn main() {}
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_as (
    ($out_struct:path, $path:literal) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path)
    });
    ($out_struct:path, $path:literal, $($args:expr),* $(,)?) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args)*])
    })
);

/// A variant of [`query_as!`][`crate::query_as!`] which does not check the input or output types. This still does parse
/// the query to ensure it's syntactically and semantically valid for the current database.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_as_unchecked (
    ($out_struct:path, $query:expr) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, checked = false)
    });

    ($out_struct:path, $query:expr, $($args:expr),* $(,)?) => ( {
        $crate::internal::sqlx_d1_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*], checked = false)
    })
);

/// A variant of [`query!`][`crate::query!`] which expects a single column from the query and evaluates to an
/// instance of [QueryScalar][crate::query::QueryScalar].
///
/// The name of the column is not required to be a valid Rust identifier, however you can still
/// use the column type override syntax in which case the column name _does_ have to be a valid
/// Rust identifier for the override to parse properly. If the override parse fails the error
/// is silently ignored (we just don't have a reliable way to tell the difference). **If you're
/// getting a different type than expected, please check to see if your override syntax is correct
/// before opening an issue.**
///
/// Wildcard overrides like in [`query_as!`][`crate::query_as!`] are also allowed, in which case the output type
/// is left up to inference.
///
/// See [`query!`][`crate::query!`] for more information.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_scalar (
    ($query:expr) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source = $query)
    );
    ($query:expr, $($args:expr),* $(,)?) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source = $query, args = [$($args)*])
    )
);

/// A variant of [`query_scalar!`][`crate::query_scalar!`] which takes a file path like
/// [`query_file!`][`crate::query_file!`].
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_scalar (
    ($path:literal) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path)
    );
    ($path:literal, $($args:expr),* $(,)?) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*])
    )
);

/// A variant of [`query_scalar!`][`crate::query_scalar!`] which does not typecheck bind parameters
/// and leaves the output type to inference.
/// The query itself is still checked that it is syntactically and semantically
/// valid for the database, that it only produces one column and that the number of bind parameters
/// is correct.
///
/// For this macro variant the name of the column is irrelevant.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_scalar_unchecked (
    ($query:expr) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source = $query, checked = false)
    );
    ($query:expr, $($args:expr),* $(,)?) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source = $query, args = [$($args)*], checked = false)
    )
);

/// A variant of [`query_file_scalar!`][`crate::query_file_scalar!`] which does not typecheck bind
/// parameters and leaves the output type to inference.
/// The query itself is still checked that it is syntactically and
/// semantically valid for the database, that it only produces one column and that the number of
/// bind parameters is correct.
///
/// For this macro variant the name of the column is irrelevant.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
macro_rules! query_file_scalar_unchecked (
    ($path:literal) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, checked = false)
    );
    ($path:literal, $($args:expr),* $(,)?) => (
        $crate::internal::sqlx_d1_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*], checked = false)
    )
);
