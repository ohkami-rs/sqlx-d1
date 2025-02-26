use ohkami::prelude::*;

#[ohkami::bindings]
struct Bindings {
    DB: ohkami::bindings::D1,
}

#[ohkami::worker]
async fn my_worker(Bindings { DB }: Bindings) -> Ohkami {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let pool = sqlx::pool::PoolOptions::<sqlx_d1::D1>::new()
        .connect_with(sqlx_d1::D1ConnectOptions::new(DB))
        .await.unwrap();

    Ohkami::new((
        "/".GET(|| async {"Hello, Cloudflare Workers!"}),
    ))
}
