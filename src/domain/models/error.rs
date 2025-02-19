// domain/models/error.rs
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AttachmentError {
    FileTooBig(usize),
    InvalidFileType(String),
    StorageError(String),
    DatabaseError(sqlx::Error),
    NotFound,
}

impl fmt::Display for AttachmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttachmentError::FileTooBig(size) => {
                write!(f, "File size {} bytes exceeds limit", size)
            }
            AttachmentError::InvalidFileType(ext) => write!(f, "Invalid file type: {}", ext),
            AttachmentError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            AttachmentError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AttachmentError::NotFound => write!(f, "Attachment not found"),
        }
    }
}

impl IntoResponse for AttachmentError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AttachmentError::FileTooBig(_) => (StatusCode::BAD_REQUEST, "File size exceeds limit"),
            AttachmentError::InvalidFileType(_) => (StatusCode::BAD_REQUEST, "Invalid file type"),
            AttachmentError::StorageError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error storing attachment",
            ),
            AttachmentError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AttachmentError::NotFound => (StatusCode::NOT_FOUND, "Attachment not found"),
        };

        (
            status,
            Json(json!({
                "status": "error",
                "message": message
            })),
        )
            .into_response()
    }
}
