use ohkami::prelude::*;

#[ohkami::bindings]
struct Bindings {
    DB: ohkami::bindings::D1,
}

#[ohkami::worker]
async fn my_worker() -> Ohkami {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // sqlx::Pool::connect_with(
    //     sqlx_d1::D1ConnectOptions::new(env, binding)
    // )

    Ohkami::new((
        "/".GET(|| async {"Hello, Cloudflare Workers!"}),
    ))
}
