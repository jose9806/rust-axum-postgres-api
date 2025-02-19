#![allow(unused)]
use axum::{
    extract::{Multipart, Path, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::fs;
use utoipa::path;
use uuid::Uuid;

use crate::{
    domain::{
        models::{attachment::AttachmentModel, error::AttachmentError},
        schemas::attachment_schema::{
            DeleteAttachmentResponse, FileDownloadResponse, UploadAttachmentSchema,
        },
    },
    infrastructure::database::AppState,
    services::attachment_service::AttachmentService,
};

/// Upload a file attachment for a specific note
///
/// This endpoint accepts a multipart form-data file upload with the following constraints:
/// - Maximum file size: 10MB
/// - Allowed file types: PDF, DOC, DOCX, TXT
/// - Files are stored with a UUID prefix for security
///
/// On successful upload, returns the attachment metadata including the generated ID
/// and original filename.
#[utoipa::path(
    post,
    path = "/api/notes/{note_id}/attachments",
    request_body(
        content_type = "multipart/form-data",
        content = inline(UploadAttachmentSchema),
        description = "File upload with maximum size of 10MB",
        example = json!({
            "file": "binary data"
        })
    ),
    responses(
        (status = 200, description = "Attachment uploaded successfully", body = AttachmentModel),
        (status = 400, description = "Invalid file type or missing file"),
        (status = 413, description = "File too large"),
        (status = 500, description = "Server error during upload")
    ),
    params(
        ("note_id" = uuid::Uuid, Path, description = "ID of the note for the attachment")
    )
)]
pub async fn upload_attachment_handler(
    Path(note_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AttachmentError> {
    let service = AttachmentService::new(app_state.db.clone());

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AttachmentError::StorageError(format!("Invalid multipart form: {}", e)))?
        .ok_or_else(|| AttachmentError::StorageError("No file provided".to_string()))?;

    let filename = field
        .file_name()
        .ok_or_else(|| AttachmentError::StorageError("No filename provided".to_string()))?
        .to_string();

    let content = field
        .bytes()
        .await
        .map_err(|e| AttachmentError::StorageError(format!("Failed to read file: {}", e)))?;

    let attachment = service
        .create_attachment(note_id, filename, content.to_vec())
        .await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "attachment": attachment
        }
    })))
}

/// List all attachments associated with a specific note
///
/// Returns a list of attachments sorted by creation date (newest first).
/// Each attachment includes metadata such as filename and creation date.
#[utoipa::path(
    get,
    path = "/api/notes/{note_id}/attachments",
    responses(
        (status = 200, description = "List of attachments retrieved successfully", body = Vec<AttachmentModel>),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Server error")
    )
)]
pub async fn list_attachments_handler(
    Path(note_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AttachmentError> {
    let service = AttachmentService::new(app_state.db.clone());
    let attachments = service.list_attachments_by_note(note_id).await?;

    Ok(Json(json!({
        "status": "success",
        "results": attachments.len(),
        "attachments": attachments
    })))
}

/// Download a specific attachment file
///
/// Downloads the file content as an attachment with its original filename.
/// The file will be served with a generic binary content type for universal compatibility.
#[utoipa::path(
    get,
    path = "/api/attachments/{attachment_id}/download",
    params(
        ("attachment_id" = uuid::Uuid, Path, description = "ID of the attachment to download")
    ),
    responses(
        (status = 200, description = "File downloaded successfully", 
         content_type = "application/octet-stream",
         body = FileDownloadResponse,
         headers(
             ("Content-Disposition" = String, description = "Attachment filename")
         )
        ),
        (status = 404, description = "Attachment not found"),
        (status = 500, description = "Server error during download")
    )
)]
pub async fn download_attachment_handler(
    Path(attachment_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AttachmentError> {
    let service = AttachmentService::new(app_state.db.clone());
    let attachment = service.get_attachment(attachment_id).await?;

    let file_content = fs::read(&attachment.file_path)
        .await
        .map_err(|e| AttachmentError::StorageError(format!("Failed to read file: {}", e)))?;

    let mut headers = HeaderMap::new();

    // Set Content-Disposition header
    let filename = attachment
        .original_filename
        .unwrap_or_else(|| "download".to_string());
    let content_disposition = format!("attachment; filename=\"{}\"", filename);
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition).unwrap(),
    );

    // Use application/octet-stream for universal compatibility
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    Ok((headers, file_content))
}

/// Delete an attachment and its associated file
///
/// Permanently removes both the database record and the stored file.
/// This operation is idempotent - calling it multiple times on the same ID
/// will not result in an error.
#[utoipa::path(
    delete,
    path = "/api/attachments/{attachment_id}",
    params(
        ("attachment_id" = uuid::Uuid, Path, description = "Unique identifier of the attachment to delete")
    ),
    responses(
        (status = 204, description = "Attachment deleted successfully", body = DeleteAttachmentResponse),
        (status = 404, description = "Attachment not found", body = DeleteAttachmentResponse),
        (status = 500, description = "Server error during deletion", body = DeleteAttachmentResponse)
    ),
    security(
        ("api_key" = [])
    )
)]
pub async fn delete_attachment_handler(
    Path(attachment_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AttachmentError> {
    let service = AttachmentService::new(app_state.db.clone());

    match service.delete_attachment(attachment_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(AttachmentError::NotFound) => Err(AttachmentError::NotFound),
        Err(e) => {
            eprintln!("Error deleting attachment {}: {}", attachment_id, e);
            Err(e)
        }
    }
}
