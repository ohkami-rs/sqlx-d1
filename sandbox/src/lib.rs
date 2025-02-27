use ohkami::prelude::*;
use ohkami::typed::status;
use sqlx_d1::{D1, D1Connection};

#[ohkami::bindings]
struct Bindings {
    DB: ohkami::bindings::D1,
}

#[derive(Serialize, sqlx_d1::FromRow)]
struct User {
    id: u32,
    name: String,
    age: Option<u8>,
}

#[derive(Deserialize)]
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

    Ohkami::new((
        Context::new(D1Connection::new(DB)),
        "/"
            .GET(async |
                Context(c): Context<'_, D1Connection>,
            | -> Result<JSON<Vec<User>>, Error> {
                let sql = "
                    SELECT id, name, age FROM users
                ";
                let users = sqlx_d1::query_as::<User>(sql)
                    .fetch_all(c)
                    .await?;

                Ok(JSON(users))
            })
            .POST(async |
                Context(c): Context<'_, D1Connection>,
                JSON(req): JSON<CreateUserRequest<'_>>,
            | -> Result<status::Created<JSON<User>>, Error> {
                let sql = "
                    INSERT INTO users (name, age) VALUES (?, ?)
                    RETURNING id
                ";
                let created_id = sqlx_d1::query_scalar::<u32>(sql)
                    .bind(req.name)
                    .bind(req.age)
                    .fetch_one(c)
                    .await?;

                Ok(status::Created(JSON(User {
                    id: created_id,
                    name: req.name.to_string(),
                    age: req.age,
                })))
            }),
        "/:id"
            .GET(async |
                id: u32,
                Context(c): Context<'_, D1Connection>,
            | -> Result<JSON<User>, Error> {
                let sql = "
                    SELECT id, name, age FROM users
                    WHERE id = ?
                ";
                let user = sqlx_d1::query_as::<User>(sql)
                    .bind(id)
                    .fetch_optional(c)
                    .await?
                    .ok_or_else(|| Error::ResourceNotFound(format!(
                        "User(id = {id})"
                    )))?;

                Ok(JSON(user))
            }),
    ))
}
