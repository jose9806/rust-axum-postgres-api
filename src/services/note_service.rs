use crate::infrastructure::repositories::note_repository;
use crate::{
    domain::models::note::NoteModel,
    domain::schemas::note_schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn list_notes(
    pool: &Pool<Postgres>,
    filters: FilterOptions,
) -> Result<Vec<NoteModel>, sqlx::Error> {
    note_repository::list_notes(pool, filters).await
}

pub async fn create_note(
    pool: &Pool<Postgres>,
    payload: CreateNoteSchema,
) -> Result<NoteModel, sqlx::Error> {
    note_repository::create_note(
        pool,
        payload.title,
        payload.content,
        payload.category,
        payload.published,
    )
    .await
}

pub async fn get_note(pool: &Pool<Postgres>, id: Uuid) -> Result<NoteModel, sqlx::Error> {
    note_repository::get_note_by_id(pool, id).await
}

pub async fn update_note(
    pool: &Pool<Postgres>,
    id: Uuid,
    payload: UpdateNoteSchema,
) -> Result<NoteModel, sqlx::Error> {
    note_repository::update_note(
        pool,
        id,
        payload.title,
        payload.content,
        payload.category,
        payload.published,
    )
    .await
}

pub async fn delete_note(pool: &Pool<Postgres>, id: Uuid) -> Result<(), sqlx::Error> {
    note_repository::delete_note(pool, id).await
}
