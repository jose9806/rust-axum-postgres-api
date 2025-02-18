// File: src/api/attachments/handler.rs

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use utoipa::path;
use uuid::Uuid;

use crate::domain::models::attachment::AttachmentModel;
use crate::infrastructure::database::AppState;
use crate::services::attachment_service;

// Directory where uploaded files are stored
static UPLOAD_DIR: &str = "uploads/";

/// Upload an attachment for a specific note.
///
/// This endpoint accepts a multipart form-data file upload.
///
/// **Path:** `/api/notes/{note_id}/attachments`
#[utoipa::path(
    post,
    path = "/api/notes/{note_id}/attachments",
    responses(
        (status = 200, description = "Attachment uploaded successfully"),
        (status = 400, description = "No file found in multipart data"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("note_id" = uuid::Uuid, Path, description = "ID of the note for the attachment")
    )
)]
pub async fn upload_attachment_handler(
    Path(note_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(e) = fs::create_dir_all(UPLOAD_DIR).await {
        let error_response =
            json!({ "status": "error", "message": format!("Cannot create uploads dir: {:?}", e) });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    if let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|s| s.to_string());
        let original_filename = filename.clone();

        let unique_filename = format!(
            "{}-{}",
            uuid::Uuid::new_v4(),
            filename.unwrap_or("file".to_string())
        );
        let file_path = format!("{}{}", UPLOAD_DIR, unique_filename);

        let bytes = field.bytes().await.unwrap();
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(&bytes).await.unwrap();

        match attachment_service::create_attachment(
            &app_state.db,
            note_id,
            file_path.clone(),
            original_filename,
        )
        .await
        {
            Ok(attachment) => {
                let response = json!({
                    "status": "success",
                    "data": {
                        "attachment": attachment
                    }
                });
                return Ok(Json(response));
            }
            Err(e) => {
                let error_response = json!({
                    "status": "error",
                    "message": format!("Cannot insert attachment: {:?}", e)
                });
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
            }
        }
    }

    // If no file field was found
    let error_response = json!({
        "status": "fail",
        "message": "No file found in multipart data",
    });
    Err((StatusCode::BAD_REQUEST, Json(error_response)))
}

/// List attachments for a given note.
///
/// **Path:** `/api/notes/{note_id}/attachments`
#[utoipa::path(
    get,
    path = "/api/notes/{note_id}/attachments",
    responses(
        (status = 200, description = "List attachments for the note", 
         body = Vec<AttachmentModel>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("note_id" = uuid::Uuid, Path, description = "ID of the note")
    )
)]
pub async fn list_attachments_handler(
    Path(note_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match attachment_service::list_attachments_by_note(&app_state.db, note_id).await {
        Ok(attachments) => {
            let response = json!({
                "status": "success",
                "results": attachments.len(),
                "attachments": attachments
            });
            Ok(Json(response))
        }
        Err(err) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error listing attachments: {:?}", err),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Download a specific attachment.
///
/// **Path:** `/api/attachments/{attachment_id}/download`
#[utoipa::path(
    get,
    path = "/api/attachments/{attachment_id}/download",
    responses(
        (status = 200, description = "Download attachment file", content_type = "application/octet-stream"),
        (status = 404, description = "Attachment not found")
    ),
    params(
        ("attachment_id" = uuid::Uuid, Path, description = "ID of the attachment")
    )
)]
pub async fn download_attachment_handler(
    Path(attachment_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<(StatusCode, Vec<u8>), (StatusCode, Json<serde_json::Value>)> {
    // Retrieve attachment info from DB
    let attachment = match attachment_service::get_attachment(&app_state.db, attachment_id).await {
        Ok(a) => a,
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": "Attachment not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    };

    // Read file from disk
    match fs::read(&attachment.file_path).await {
        Ok(file_bytes) => Ok((StatusCode::OK, file_bytes)),
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": "File not found on disk"
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Delete a specific attachment.
///
/// **Path:** `/api/attachments/{attachment_id}`
#[utoipa::path(
    delete,
    path = "/api/attachments/{attachment_id}",
    responses(
        (status = 204, description = "Attachment deleted successfully"),
        (status = 404, description = "Attachment not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("attachment_id" = uuid::Uuid, Path, description = "ID of the attachment to delete")
    )
)]
pub async fn delete_attachment_handler(
    Path(attachment_id): Path<Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Retrieve attachment info first
    let attachment = match attachment_service::get_attachment(&app_state.db, attachment_id).await {
        Ok(a) => a,
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": "Attachment not found"
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    };

    // Remove DB record
    match attachment_service::delete_attachment(&app_state.db, attachment_id).await {
        Ok(_) => {
            // Remove the file from disk
            let _ = fs::remove_file(&attachment.file_path).await;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(err) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error deleting attachment: {:?}", err),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
