<div align="center">
    <h1>SQLx-D1</h1>
    [SQLx](https://github.com/launchbadge/sqlx) for [Cloudflare D1](https://developers.cloudflare.com/d1).
</div>

<br>

SQLx-D1 realizes "SQLx for Cloudflare D1" with _**compile-time SQL verification**_ in Rust Cloudflare development !

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
