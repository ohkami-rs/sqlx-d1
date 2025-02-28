<div align="center">
    <h1>SQLx-D1</h1>
    <a href="https://github.com/launchbadge/sqlx">SQLx</a> for <a href="https://developers.cloudflare.com/d1">Cloudflare D1</a>.
</div>

<br>

SQLx-D1 realizes "SQLx for Cloudflare D1"  _**withcompile-time SQL verification**_ in Rust Cloudflare development !

## Background

*Miniflare's local D1 emulator is, essentially, just an `.sqlite` file.*

This fact has been brought a lot of Rustaceans trying `sqlx` with `sqlite` feature for D1, but it's impossible because `sqlx-sqlite` contains a *native dependency* of SQLite driver.

SQLx-D1 works around this by loading `sqlx-sqlite` **only in macro context** and just providing a conversion layer between D1 and SQLx **in library context**. 

## Unsupported features

- Transaction ( let's wait for Cloudflare's side to support transation on D1 ! )
- Connection pool ( `sqlx::Pool` internally requires Rust async runtime (tokio / asycn-std) and time implemetation on the WASM runtime which is not done on Cloudflare Workers )
    - alternatively, `&sqlx_d1::D1Connection` implements `Executor`, not only `&mut` one.

## TODO features

- offline mode
- chrono integration

## Example

```toml
# Cargo.toml

[dependencies]
sqlx_d1 = { git = "https://github.com/ohkami-rs/sqlx-d1" }
worker = { version = "0.5", features = ["d1"] }
```
```toml
# wrangler.toml

[[d1_database]]
binding = "DB"
database_name = "..."
database_id = "..."
```
```rust
// src/lib.rs

#[worker::event(fetch)]
async fn main(
    req: worker::Request,
    env: worker::Env,
    ctx: worker::Context,
) -> worker::Result<worker::Response> {
    let d1 = env.d1("DB")?;
    let conn = sqlx_d1::D1Connection::new(d1);

    sqlx::query!("INSERT INTO users (name, age) VALUES (?, ?)", "dog", 42)
        .execute(&conn)
        .await
        .map_err(|e| worker::Error::Rust(e.to_string()))?;

    worker::Response::ok("Hello, sqlx_d1 !")
}
```

## LICENSE

SQLx-D1 is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/sqlx-d1/blob/main/LICENSE) or https://opensource.org/licenses/MIT ) .
