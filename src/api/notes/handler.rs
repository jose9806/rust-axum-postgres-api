use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::domain::schemas::note_schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema};
use crate::infrastructure::database::AppState;
use crate::services::note_service;

pub async fn health_checker_handler() -> impl IntoResponse {
    Json(json!({
        "status": "success",
        "message": "Simple CRUD API with Rust, Axum, SQLx, and Postgres"
    }))
}

pub async fn list_notes_handler(
    State(app_state): State<Arc<AppState>>,
    Query(filters): Query<FilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
