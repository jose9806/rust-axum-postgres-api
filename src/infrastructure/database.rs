use sqlx::{Pool, Postgres};

pub struct AppState {
    pub db: Pool<Postgres>,
}

pub async fn init_db_pool(database_url: &str) -> Pool<Postgres> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to the database")
}
