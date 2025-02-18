// File: src/api/notes/router.rs

use crate::api::notes::handler::{
    create_note_handler, delete_note_handler, edit_note_handler, get_note_handler,
    health_checker_handler, list_notes_handler,
};
use crate::infrastructure::database::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_notes_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/", post(create_note_handler).get(list_notes_handler))
        .route(
            "/api/{id}",
            get(get_note_handler)
                .patch(edit_note_handler)
                .delete(delete_note_handler),
        )
        .with_state(app_state)
}
