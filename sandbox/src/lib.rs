#[cfg(feature = "DEBUG")]
pub mod readme_sample;

use ohkami::prelude::*;
use ohkami::typed::status;
use sqlx_d1::D1Connection;
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

#[ohkami::bindings]
struct Bindings {
    DB: ohkami::bindings::D1,
}

#[derive(Serialize, sqlx_d1::FromRow)]
struct User {
    id: i64,
    uuid: Option<HyphenatedUuid>,
    name: String,
    age: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct CreateUserRequest<'req> {
    name: &'req str,
    age: Option<u8>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Error from D1: {0}")]
    D1(#[from] sqlx_d1::Error),
    #[error("Error not found {0}")]
    ResourceNotFound(String),
}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        worker::console_error!("{self}");
        match self {
            Self::D1(_) => Response::InternalServerError(),
            Self::ResourceNotFound(_) => Response::NotFound(),
        }
    }
}

#[ohkami::worker]
async fn my_worker(Bindings { DB }: Bindings) -> Ohkami {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let d1_connection = sqlx_d1::D1ConnectOptions::new(DB)
        .foreign_keys(true)
        .connect()
        .await
        .expect("Failed to connect to D1");

    Ohkami::new((
        Context::new(d1_connection),
        
        "/"
            .GET(async |
                Context(c): Context<'_, D1Connection>,
            | -> Result<JSON<Vec<User>>, Error> {
                let users = sqlx_d1::query_as!(User, "
                    SELECT id, uuid, name, age FROM users
                ")
                    .fetch_all(c)
                    .await?;

                Ok(JSON(users))
            })
            .POST(async |
                Context(c): Context<'_, D1Connection>,
                JSON(req): JSON<CreateUserRequest<'_>>,
            | -> Result<status::Created<JSON<User>>, Error> {
                let uuid = HyphenatedUuid::from_uuid(
                    uuid::Uuid::parse_str(
                        &js::randomUUID()
                    ).unwrap()
                );

                let created_id = sqlx_d1::query_scalar!("
                    INSERT INTO users (uuid, name, age) VALUES (?, ?, ?)
                    RETURNING id
                ", uuid, req.name, req.age).fetch_one(c).await?;

                Ok(status::Created(JSON(User {
                    id: created_id,
                    uuid: Some(uuid.into()),
                    name: req.name.to_string(),
                    age: req.age.map(|a| a.try_into().ok()).flatten(),
                })))
            }),
        "/:id"
            .GET(async |
                id: u32,
                Context(c): Context<'_, D1Connection>,
            | -> Result<JSON<User>, Error> {
                let user_record = sqlx_d1::query!("
                    SELECT uuid, name, age FROM users
                    WHERE id = ?
                ", id).fetch_optional(c).await?
                    .ok_or_else(|| Error::ResourceNotFound(format!(
                        "User(id = {id})"
                    )))?;

                Ok(JSON(User {
                    id: id.into(),
                    uuid: user_record.uuid.map(|uuid| {
                        HyphenatedUuid::from_uuid(
                            uuid::Uuid::parse_str(&uuid).unwrap()
                        )
                    }),
                    name: user_record.name,
                    age: user_record.age,
                }))
            }),
    ))
}
