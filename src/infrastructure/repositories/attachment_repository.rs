use crate::domain::models::attachment::AttachmentModel;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

pub async fn create_attachment(
    pool: &Pool<Postgres>,
    note_id: Uuid,
    file_path: String,
    original_filename: Option<String>,
) -> Result<AttachmentModel, Error> {
    let attachment = sqlx::query_as!(
        AttachmentModel,
        r#"
        INSERT INTO attachments (note_id, file_path, original_filename)
        VALUES ($1, $2, $3)
        RETURNING 
            id, 
            note_id, 
            file_path, 
            original_filename, 
            created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        "#,
        note_id,
        file_path,
        original_filename
    )
    .fetch_one(pool)
    .await?;

    Ok(attachment)
}

pub async fn list_attachments_by_note(
    pool: &Pool<Postgres>,
    note_id: Uuid,
) -> Result<Vec<AttachmentModel>, Error> {
    let attachments = sqlx::query_as!(
        AttachmentModel,
        r#"
        SELECT
            id,
            note_id,
            file_path,
            original_filename,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM attachments
        WHERE note_id = $1
        ORDER BY created_at DESC
        "#,
        note_id
    )
    .fetch_all(pool)
    .await?;

    Ok(attachments)
}

pub async fn get_attachment(
    pool: &Pool<Postgres>,
    attachment_id: Uuid,
) -> Result<AttachmentModel, Error> {
    let attachment = sqlx::query_as!(
        AttachmentModel,
        r#"
        SELECT
            id,
            note_id,
            file_path,
            original_filename,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>"
        FROM attachments
        WHERE id = $1
        "#,
        attachment_id
    )
    .fetch_one(pool)
    .await?;

    Ok(attachment)
}

pub async fn delete_attachment(pool: &Pool<Postgres>, attachment_id: Uuid) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM attachments
        WHERE id = $1
        "#,
        attachment_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
