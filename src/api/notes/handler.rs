use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use utoipa::path;

use crate::domain::models::note::NoteModel;
use crate::domain::schemas::note_schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema};
use crate::infrastructure::database::AppState;
use crate::services::note_service;

/// Health checker endpoint.
///
/// **Path:** `/api/healthchecker`
#[utoipa::path(
    get,
    path = "/api/healthchecker",
    responses(
        (status = 200, description = "Health check successful")
    )
)]
pub async fn health_checker_handler() -> impl IntoResponse {
    Json(json!({
        "status": "success",
        "message": "Simple CRUD API with Rust, Axum, SQLx, and Postgres"
    }))
}

/// List all notes with optional filtering.
///
/// **Path:** `/api/`
///
/// Query parameters:
/// - `limit`: Limit number of results.
/// - `page`: Page number.
/// - `tags`: List of tags to filter by.
#[utoipa::path(
    get,
    path = "/api/",
    responses(
        (status = 200, description = "List notes", body = Vec<NoteModel>),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("limit" = Option<usize>, Query, description = "Limit number of notes"),
        ("page" = Option<usize>, Query, description = "Page number"),
        ("tags" = Option<Vec<String>>, Query, description = "Filter notes by tags")
    )
)]
pub async fn list_notes_handler(
    State(app_state): State<Arc<AppState>>,
    Query(mut filters): Query<FilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    filters.tags = filters.tags.iter().map(|t| t.to_lowercase()).collect();
    match note_service::list_notes(&app_state.db, filters).await {
        Ok(notes) => {
            let response = json!({
                "status": "success",
                "results": notes.len(),
                "notes": notes
            });
            Ok(Json(response))
        }
        Err(err) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error fetching notes: {:?}", err),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Create a new note.
///
/// **Path:** `/api/`
///
/// Request body: [CreateNoteSchema](crate::domain::schemas::note_schema::CreateNoteSchema)
#[utoipa::path(
    post,
    path = "/api/",
    request_body = CreateNoteSchema,
    responses(
        (status = 201, description = "Note created", body = NoteModel),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_note_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match note_service::create_note(&app_state.db, payload).await {
        Ok(note) => {
            let response = json!({
                "status": "success",
                "data": { "note": note }
            });
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(err) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error creating note: {:?}", err),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Retrieve a single note by ID.
///
/// **Path:** `/api/{id}`
#[utoipa::path(
    get,
    path = "/api/{id}",
    responses(
        (status = 200, description = "Note found", body = NoteModel),
        (status = 404, description = "Note not found")
    ),
    params(
        ("id" = uuid::Uuid, Path, description = "ID of the note")
    )
)]
pub async fn get_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match note_service::get_note(&app_state.db, id).await {
        Ok(note) => {
            let response = json!({
                "status": "success",
                "data": { "note": note }
            });
            Ok(Json(response))
        }
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

/// Update an existing note.
///
/// **Path:** `/api/{id}`
///
/// Request body: [UpdateNoteSchema](crate::domain::schemas::note_schema::UpdateNoteSchema)
#[utoipa::path(
    patch,
    path = "/api/{id}",
    request_body = UpdateNoteSchema,
    responses(
        (status = 200, description = "Note updated", body = NoteModel),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = uuid::Uuid, Path, description = "ID of the note")
    )
)]
pub async fn edit_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match note_service::update_note(&app_state.db, id, payload).await {
        Ok(note) => {
            let response = json!({
                "status": "success",
                "data": { "note": note }
            });
            Ok(Json(response))
        }
        Err(err) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error updating note: {:?}", err),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Delete a note by ID.
///
/// **Path:** `/api/{id}`
#[utoipa::path(
    delete,
    path = "/api/{id}",
    responses(
        (status = 204, description = "Note deleted"),
        (status = 404, description = "Note not found")
    ),
    params(
        ("id" = uuid::Uuid, Path, description = "ID of the note")
    )
)]
pub async fn delete_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match note_service::delete_note(&app_state.db, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}
