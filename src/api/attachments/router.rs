use crate::api::attachments::handler::{
    delete_attachment_handler, download_attachment_handler, list_attachments_handler,
    upload_attachment_handler,
};
use crate::infrastructure::database::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

pub fn create_attachments_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/notes/{note_id}/attachments",
            post(upload_attachment_handler),
        )
        .route(
            "/api/notes/{note_id}/attachments",
            get(list_attachments_handler),
        )
        .route(
            "/api/attachments/{attachment_id}/download",
            get(download_attachment_handler),
        )
        .route(
            "/api/attachments/{attachment_id}",
            delete(delete_attachment_handler),
        )
        .with_state(app_state)
}
