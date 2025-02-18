use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use api::attachments::router::create_attachments_routes;
use api::notes::router::create_notes_routes;
use api::ApiDoc;
use infrastructure::database::{init_db_pool, AppState};

use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

mod api;
mod domain;
mod infrastructure;
mod services;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = init_db_pool(&database_url).await;
    let app_state = Arc::new(AppState { db: db_pool });

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let openapi_spec = ApiDoc::openapi();

    let docs_router = SwaggerUi::new("/docs").urls(vec![(
        Url::new("API Docs", "/api-docs.json"),
        openapi_spec.clone(),
    )]);

    let app = axum::Router::new()
        .merge(create_notes_routes(app_state.clone()))
        .merge(create_attachments_routes(app_state.clone()))
        .merge(docs_router)
        .layer(cors);

    println!("🚀 Server running on http://0.0.0.0:8000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!(
        "🚀 Server running on http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
