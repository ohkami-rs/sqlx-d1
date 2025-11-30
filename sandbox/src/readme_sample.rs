pub async fn main(
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
