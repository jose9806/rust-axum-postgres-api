use std::path::Path;
use tokio::fs;
use uuid::Uuid;

use sqlx::{Pool, Postgres};

use crate::domain::models::attachment::AttachmentModel;
use crate::domain::models::error::AttachmentError;
use crate::infrastructure::repositories::attachment_repository;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const ALLOWED_EXTENSIONS: [&str; 9] = [
    "pdf", "doc", "docx", "txt", // Document formats
    "jpg", "jpeg", "png", "gif", "webp", // Image formats
];
const UPLOAD_DIR: &str = "uploads";

pub struct AttachmentService {
    pool: Pool<Postgres>,
}

impl AttachmentService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    async fn validate_file(&self, filename: &str, size: usize) -> Result<(), AttachmentError> {
        if size > MAX_FILE_SIZE {
            return Err(AttachmentError::FileTooBig(size));
        }

        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            return Err(AttachmentError::InvalidFileType(format!(
                "File type '.{}' is not supported. Allowed types are: {}",
                extension,
                ALLOWED_EXTENSIONS.join(", ")
            )));
        }

        Ok(())
    }

    async fn ensure_upload_dir(&self) -> Result<(), AttachmentError> {
        fs::create_dir_all(UPLOAD_DIR)
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))
    }

    pub async fn create_attachment(
        &self,
        note_id: Uuid,
        filename: String,
        content: Vec<u8>,
    ) -> Result<AttachmentModel, AttachmentError> {
        self.validate_file(&filename, content.len()).await?;
        self.ensure_upload_dir().await?;

        let unique_filename = format!("{}-{}", Uuid::new_v4(), filename);
        let file_path = format!("{}/{}", UPLOAD_DIR, unique_filename);

        // Write file to disk
        if let Err(e) = fs::write(&file_path, &content).await {
            return Err(AttachmentError::StorageError(e.to_string()));
        }

        // Create database record
        match attachment_repository::create_attachment(
            &self.pool,
            note_id,
            file_path.clone(),
            Some(filename),
        )
        .await
        {
            Ok(attachment) => Ok(attachment),
            Err(e) => {
                // Clean up file if database insert fails
                let _ = fs::remove_file(&file_path).await;
                Err(AttachmentError::DatabaseError(e))
            }
        }
    }

    pub async fn delete_attachment(&self, attachment_id: Uuid) -> Result<(), AttachmentError> {
        // First, get the attachment details and verify it exists
        let attachment = attachment_repository::get_attachment(&self.pool, attachment_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AttachmentError::NotFound,
                e => AttachmentError::DatabaseError(e),
            })?;

        // Verify the file exists before attempting deletion
        if !Path::new(&attachment.file_path).exists() {
            return Err(AttachmentError::StorageError(
                "File not found on disk".to_string(),
            ));
        }

        // Delete the database record
        attachment_repository::delete_attachment(&self.pool, attachment_id)
            .await
            .map_err(AttachmentError::DatabaseError)?;

        // Delete the file from disk
        match fs::remove_file(&attachment.file_path).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If file deletion fails, log the error but consider the operation successful
                // since the database record is already removed
                eprintln!(
                    "Warning: Failed to delete file {}: {}",
                    attachment.file_path, e
                );
                Ok(())
            }
        }
    }
    pub async fn get_attachment(
        &self,
        attachment_id: Uuid,
    ) -> Result<AttachmentModel, AttachmentError> {
        let attachment = attachment_repository::get_attachment(&self.pool, attachment_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AttachmentError::NotFound,
                e => AttachmentError::DatabaseError(e),
            })?;

        // Verify file exists on disk
        if !Path::new(&attachment.file_path).exists() {
            return Err(AttachmentError::StorageError(
                "File exists in database but not on disk".to_string(),
            ));
        }

        Ok(attachment)
    }

    pub async fn list_attachments_by_note(
        &self,
        note_id: Uuid,
    ) -> Result<Vec<AttachmentModel>, AttachmentError> {
        attachment_repository::list_attachments_by_note(&self.pool, note_id)
            .await
            .map_err(AttachmentError::DatabaseError)
    }
}
