pub mod attachments;
pub mod notes;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::notes::handler::health_checker_handler,
        crate::api::notes::handler::list_notes_handler,
        crate::api::notes::handler::create_note_handler,
        crate::api::notes::handler::get_note_handler,
        crate::api::notes::handler::edit_note_handler,
        crate::api::notes::handler::delete_note_handler,
        crate::api::attachments::handler::upload_attachment_handler,
        crate::api::attachments::handler::list_attachments_handler,
        crate::api::attachments::handler::download_attachment_handler,
        crate::api::attachments::handler::delete_attachment_handler
    ),
    components(
        // Include your data models and schemas here.
        // For example, if your NoteModel and AttachmentModel are located in `crate::domain::models`,
        // and your note schemas are in `crate::domain::schemas::note_schema`,
        // you might do:
        // schemas(crate::domain::models::note_model::NoteModel, crate::domain::models::attachment_model::AttachmentModel,
        //         crate::domain::schemas::note_schema::CreateNoteSchema, crate::domain::schemas::note_schema::UpdateNoteSchema,
        //         crate::domain::schemas::note_schema::FilterOptions)
    ),
    info(
        title = "Plus Notes documentation",
        description = "An API for managing notes and attachments",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;
