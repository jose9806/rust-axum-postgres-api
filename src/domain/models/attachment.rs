use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize, ToSchema)]
pub struct AttachmentModel {
    pub id: Uuid,
    pub note_id: Uuid,
    pub file_path: String,
    pub original_filename: Option<String>,
    pub created_at: DateTime<Utc>,
}
