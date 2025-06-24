<div align="center">
    <h1>SQLx-D1</h1>
    <a href="https://github.com/launchbadge/sqlx">SQLx</a> for <a href="https://developers.cloudflare.com/d1">Cloudflare D1</a>.
</div>

<br>

SQLx-D1 realizes "SQLx for Cloudflare D1" _**with compile-time SQL verification**_ in Rust Cloudflare development !

<div align="right">
    <a href="https://github.com/ohkami-rs/sqlx-d1/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/sqlx-d1.svg" /></a>
    <a href="https://github.com/ohkami-rs/sqlx-d1/actions"><img alt="build check status" src="https://github.com/ohkami-rs/sqlx-d1/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/sqlx-d1"><img alt="crates.io" src="https://img.shields.io/crates/v/sqlx-d1" /></a>
</div>

## Background

*Miniflare's local D1 emulator is, essentially, just a `.sqlite` file.*

This fact has been brought a lot of Rustaceans trying `sqlx` with `sqlite` feature for D1, but it's impossible because:

- `sqlx-sqlite` contains a *native dependency* of SQLite driver.
- actual D1 itself doesn't expose raw database interface.

SQLx-D1 works around them by loading `sqlx-sqlite` **only in macro context** and just providing a conversion layer between D1 and SQLx **in library context**. 

## Features

- SQLx interface for Cloudflare D1
- Batteries included, `sqlx` is not needed in dependencies
- Compile-time SQL verification
    - by miniflare's local D1 emulator ( internally using `sqlx-sqlite` )
    - by query caches in `.sqlx` directory ( offline mode; `cargo sqlx prepare` )
- No environment variable or `.env` file is needed
    - D1 emulator's location is fixed to `.wrangler/state/v3/d1/miniflare-D1DatabaseObject`
    - falling back to offline mode when it doesn't exist and `.sqlx` directory exists

## Unsupported features

- Transaction
    - Let's wait for Cloudflare's side to support transation on D1 !
- Connection pool ( `sqlx::Pool` internally requires Rust async runtime (tokio / asycn-std) and time implemetation of WASM runtime which is not done on Cloudflare Workers )
    - alternatively, `&sqlx_d1::D1Connection` implements `Executor`, not only `&mut` one.
- derive `Type`, `Encode`, `Decode`
    - maybe added if requested
    - workaround if needed: add `sqlx` to dependencies and use its ones

## Example

```toml
# Cargo.toml

[dependencies]
sqlx-d1 = { version = "0.2.0", features = ["macros"] }
worker = { version = "0.6", features = ["d1"] }
serde = { version = "1.0", features = ["derive"] }
```
```sh
wrangler d1 create <DATABASE_NAME> # prints <DATABASE_ID>
```
```toml
# wrangler.toml

[[d1_databases]]
binding = "DB"
database_name = "<DATABASE_NAME>"
database_id = "<DATABASE_ID>"
```
```sh
wrangler d1 migrations create DB 'schema'
```
```sql
-- migrations/0001_schema.sql

CREATE TABLE users (
    id   INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    age  INTEGER
);
```
```sh
wrangler d1 migrations apply DB --local
```
```rust
// src/lib.rs

#[worker::event(fetch)]
async fn main(
    mut req: worker::Request,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<worker::Response> {
    let d1 = env.d1("DB")?;
    let conn = sqlx_d1::D1Connection::new(d1);

    #[derive(serde::Deserialize)]
    struct CreateUser {
        name: String,
        age: Option<u8>,
    }

    let req = req.json::<CreateUser>().await?;

    let id = sqlx_d1::query!(
        "
        INSERT INTO users (name, age) VALUES (?, ?)
        RETURNING id
        ",
            req.name,
            req.age
        )
        .fetch_one(&conn)
        .await
        .map_err(|e| worker::Error::RustError(e.to_string()))?
        .id;

    worker::Response::ok(format!("Your id is {id}!"))
}
```

## LICENSE

SQLx-D1 is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/sqlx-d1/blob/main/LICENSE) or https://opensource.org/licenses/MIT ) .
