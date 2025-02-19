use axum::Router;
use dotenv::dotenv;
use rust_axum_postgres_api::infrastructure::database::init_db_pool;
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;

pub async fn setup_database() -> Pool<Postgres> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let pool = init_db_pool(&database_url).await;
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

pub async fn teardown_database(pool: &Pool<Postgres>) {
    sqlx::query("TRUNCATE notes CASCADE")
        .execute(pool)
        .await
        .unwrap();
}

pub async fn start_test_server(app: Router) -> u16 {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    port
}
