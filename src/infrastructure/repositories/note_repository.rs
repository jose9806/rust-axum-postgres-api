use crate::domain::models::note::NoteModel;
use crate::domain::schemas::note_schema::FilterOptions;
use sqlx::{Error, Pool, Postgres};
use uuid::Uuid;

pub async fn list_notes(
    pool: &Pool<Postgres>,
    filters: FilterOptions,
) -> Result<Vec<NoteModel>, Error> {
    let limit = filters.limit.unwrap_or(10) as i64;
    let offset = (filters.page.unwrap_or(1) - 1) * limit as usize;

    let notes = sqlx::query_as!(
        NoteModel,
        r#"
        SELECT
            id,
            title,
            content,
            category,
            COALESCE(tags, '{}')::text[] as "tags!: Vec<String>",
            published,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM notes
        ORDER BY id
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(notes)
}

pub async fn create_note(
    pool: &Pool<Postgres>,
    title: String,
    content: String,
    category: Option<String>,
    published: Option<bool>,
    tags: Vec<String>,
) -> Result<NoteModel, Error> {
    let note = sqlx::query_as!(
        NoteModel,
        r#"
        INSERT INTO notes (title, content, category, published, tags)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            title,
            content,
            category,
            COALESCE(tags, '{}')::text[] as "tags!: Vec<String>",
            published,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        "#,
        title,
        content,
        category,
        published.unwrap_or(false),
        &tags
    )
    .fetch_one(pool)
    .await?;

    Ok(note)
}

pub async fn get_note_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<NoteModel, Error> {
    let note = sqlx::query_as!(
        NoteModel,
        r#"
        SELECT
            id,
            title,
            content,
            category,
            COALESCE(tags, '{}')::text[] as "tags!: Vec<String>",
            published,
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        FROM notes
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(note)
}

pub async fn update_note(
    pool: &Pool<Postgres>,
    id: Uuid,
    title: Option<String>,
    content: Option<String>,
    category: Option<String>,
    published: Option<bool>,
    tags: Option<Vec<String>>,
) -> Result<NoteModel, Error> {
    let note_opt = sqlx::query_as!(
        NoteModel,
        r#"
        UPDATE notes
        SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            category = COALESCE($3, category),
            published = COALESCE($4, published),
            tags = COALESCE($5, tags),
            updated_at = NOW()
        WHERE id = $6
        RETURNING
            id,
            title,
            content,
            category,
            published,
            COALESCE(tags, '{}')::text[] as "tags!: Vec<String>",
            created_at as "created_at!: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
        "#,
        title,
        content,
        category,
        published,
        tags.as_ref().map(|v| v.as_slice()),
        id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(note) = note_opt {
        Ok(note)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

pub async fn delete_note(pool: &Pool<Postgres>, id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM notes
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
