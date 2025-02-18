use crate::domain::models::attachment::AttachmentModel;
use crate::infrastructure::repositories::attachment_repository;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

pub async fn create_attachment(
    pool: &Pool<Postgres>,
    note_id: Uuid,
    file_path: String,
    original_filename: Option<String>,
) -> Result<AttachmentModel, Error> {
    attachment_repository::create_attachment(pool, note_id, file_path, original_filename).await
}

pub async fn list_attachments_by_note(
    pool: &Pool<Postgres>,
    note_id: Uuid,
) -> Result<Vec<AttachmentModel>, Error> {
    attachment_repository::list_attachments_by_note(pool, note_id).await
}

pub async fn get_attachment(
    pool: &Pool<Postgres>,
    attachment_id: Uuid,
) -> Result<AttachmentModel, Error> {
    attachment_repository::get_attachment(pool, attachment_id).await
}

pub async fn delete_attachment(pool: &Pool<Postgres>, attachment_id: Uuid) -> Result<(), Error> {
    attachment_repository::delete_attachment(pool, attachment_id).await
}
