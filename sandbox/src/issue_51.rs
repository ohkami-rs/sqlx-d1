pub async fn sample(
    conn: &sqlx_d1::D1Connection,
    uuid: &str,
    name: &str,
    age: Option<u8>,
) -> worker::Result<()> {
    let _query_insertinto_returning_fetch = sqlx_d1::query!(
        "
        INSERT INTO users (uuid, name, age) VALUES (?, ?, ?)
        RETURNING id
        ",
        uuid,
        name,
        age
    )
    .fetch_one(conn)
    .await
    .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let _query_insertinto_noreturning_execute = sqlx_d1::query!(
        "
        INSERT INTO users (uuid, name, age) VALUES (?, ?, ?)
        ",
        uuid,
        name,
        age
    )
    .execute(conn)
    .await
    .map_err(|e| worker::Error::RustError(e.to_string()))?;

    Ok(())
}
