#[cfg(feature = "DEBUG")]
pub mod readme_sample;

use serde::{Deserialize, Serialize};
use uuid::fmt::Hyphenated as HyphenatedUuid;

mod js {
    //! `uuid::Uuid::new_v4()` is not available on Cloudflare Workers,
    //! so we use JavaScript's `crypto.randomUUID` instead.

    use worker::wasm_bindgen;

    #[wasm_bindgen::prelude::wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = crypto)]
        pub fn randomUUID() -> String;
    }
}

#[derive(Serialize, sqlx_d1::FromRow)]
struct User {
    id: i64,
    uuid: Option<HyphenatedUuid>,
    name: String,
    age: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct CreateUserRequest {
    name: String,
    age: Option<u8>,
}

// #[derive(Debug, thiserror::Error)]
// enum Error {
//     #[error("Error from D1: {0}")]
//     D1(#[from] sqlx_d1::Error),
//     #[error("Error not found {0}")]
//     ResourceNotFound(String),
// }
// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         worker::console_error!("{self}");
//         match self {
//             Self::D1(_) => Response::InternalServerError(),
//             Self::ResourceNotFound(_) => Response::NotFound(),
//         }
//     }
// }

#[worker::event(fetch)]
async fn main(
    req: worker::Request,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<worker::Response> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let d1 = env.d1("DB")?;
    let conn = sqlx_d1::D1Connection::new(d1);

    worker::Router::new()
        .get_async("/", |_req, _ctx| {
            let conn = conn.clone();
            async move {
                let users = sqlx_d1::query_as!(User, "
                    SELECT id, uuid, name, age FROM users
                ")
                    .fetch_all(&conn)
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;

                worker::Response::from_json(&users)
            }
        })
        .post_async("/", |mut req, _ctx| {
            let conn = conn.clone();
            async move {
                let CreateUserRequest { name, age } = req.json().await?;

                let uuid = HyphenatedUuid::from_uuid(
                    uuid::Uuid::parse_str(&js::randomUUID()).unwrap()
                );

                let created_id = sqlx_d1::query_scalar!("
                    INSERT INTO users (uuid, name, age) VALUES (?, ?, ?)
                    RETURNING id
                ", uuid, name, age)
                    .fetch_one(&conn)
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;

                worker::Response::from_json(&User {
                    id: created_id,
                    uuid: Some(uuid.into()),
                    name: name.to_string(),
                    age: age.map(|a| a.into()),
                }).map(|res| res.with_status(201))
            }
        })
        .get_async("/:id", |_req, ctx| {
            let conn = conn.clone();
            async move {
                let id: u32 = ctx.param("id")
                    .ok_or_else(|| worker::Error::RustError("Missing id parameter".to_string()))?
                    .parse()
                    .map_err(|_| worker::Error::RustError("Invalid user ID".to_string()))?;

                let user_record = sqlx_d1::query!("
                    SELECT uuid, name, age FROM users
                    WHERE id = ?
                ", id)
                    .fetch_optional(&conn)
                    .await
                    .map_err(|e| worker::Error::RustError(e.to_string()))?
                    .ok_or_else(|| worker::Error::RustError(format!("User(id = {id}) not found")))?;

                let uuid = user_record.uuid.map(|uuid| {
                    HyphenatedUuid::from_uuid(
                        uuid::Uuid::parse_str(&uuid).unwrap()
                    )
                });

                let user = User {
                    id: id.into(),
                    uuid,
                    name: user_record.name,
                    age: user_record.age,
                };

                worker::Response::from_json(&user)
            }
        })
        .run(req, env)
        .await
}
