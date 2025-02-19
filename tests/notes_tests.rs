use reqwest::{Client, StatusCode};
use serde_json::json;
use std::sync::Arc;

use rust_axum_postgres_api::{
    api::notes::router::create_notes_routes, infrastructure::database::AppState,
};

mod common;
use common::{setup_database, start_test_server, teardown_database};

#[tokio::test]
async fn test_health_checker() {
    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });
    let app = create_notes_routes(app_state);
    let port = start_test_server(app).await;
    let client = Client::new();

    let response = client
        .get(format!("http://localhost:{}/api/healthchecker", port))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    teardown_database(&db_pool).await;
}

#[tokio::test]
async fn test_create_note() {
    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });
    let app = create_notes_routes(app_state);
    let port = start_test_server(app).await;
    let client = Client::new();

    let payload = json!({
        "title": "Test Note",
        "content": "Test Content",
        "category": "testing",
        "published": false,
        "tags": ["tag1", "tag2"]
    });

    let response = client
        .post(format!("http://localhost:{}/api/", port))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let response_json: serde_json::Value = response.json().await.unwrap();
    let note = &response_json["data"]["note"];
    assert_eq!(note["title"], "Test Note");
    assert_eq!(note["content"], "Test Content");
    teardown_database(&db_pool).await;
}

#[tokio::test]
async fn test_get_note() {
    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });
    let app = create_notes_routes(app_state);
    let port = start_test_server(app).await;
    let client = Client::new();

    // Create a note using POST /api/
    let note_payload = serde_json::json!({
         "title": "Test Get Note",
         "content": "Content for get note",
         "category": "test",
         "published": true,
         "tags": ["tag1"]
    });
    let create_response = client
        .post(format!("http://localhost:{}/api/", port))
        .json(&note_payload)
        .send()
        .await
        .expect("Failed to create note");

    // Verify note creation returned 201 Created.
    assert_eq!(
        create_response.status(),
        axum::http::StatusCode::CREATED,
        "Failed to create note. Response: {}",
        create_response.text().await.unwrap_or_default()
    );

    let create_json = create_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse create note response");
    let note_id = create_json["data"]["note"]["id"]
        .as_str()
        .expect("Missing note id");

    // Now, retrieve the note using GET /api/{id}
    let get_response = client
        .get(format!("http://localhost:{}/api/{}", port, note_id))
        .send()
        .await
        .expect("Failed to get note");

    assert_eq!(
        get_response.status(),
        axum::http::StatusCode::OK,
        "Get note failed. Response: {}",
        get_response.text().await.unwrap_or_default()
    );

    teardown_database(&db_pool).await;
}

#[tokio::test]
async fn test_update_note() {
    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });
    let app = create_notes_routes(app_state);
    let port = start_test_server(app).await;
    let client = Client::new();

    // Create a note using POST /api/
    let note_payload = serde_json::json!({
         "title": "Test Note to Update",
         "content": "Original content",
         "category": "test",
         "published": false,
         "tags": []
    });
    let create_response = client
        .post(format!("http://localhost:{}/api/", port))
        .json(&note_payload)
        .send()
        .await
        .expect("Failed to create note");

    // Verify note creation returns 201 Created.
    assert_eq!(
        create_response.status(),
        axum::http::StatusCode::CREATED,
        "Failed to create note. Response: {}",
        create_response.text().await.unwrap_or_default()
    );

    let create_json = create_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse create note response");
    let note_id = create_json["data"]["note"]["id"]
        .as_str()
        .expect("Missing note id");

    // Prepare an update payload
    let update_payload = serde_json::json!({
         "title": "Updated Title",
         "content": "Updated content",
         "category": "updated category",
         "published": true,
         "tags": ["updated"]
    });

    // Update the note using PATCH /api/{id}
    let update_response = client
        .patch(format!("http://localhost:{}/api/{}", port, note_id))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to update note");

    assert_eq!(
        update_response.status(),
        axum::http::StatusCode::OK,
        "Update note failed. Response: {}",
        update_response.text().await.unwrap_or_default()
    );

    teardown_database(&db_pool).await;
}

#[tokio::test]
async fn test_delete_note() {
    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });
    let app = create_notes_routes(app_state);
    let port = start_test_server(app).await;
    let client = Client::new();

    let payload = json!({
        "title": "Delete Test",
        "content": "Content",
        "category": "test",
        "published": true,
        "tags": []
    });
    let create_response = client
        .post(format!("http://localhost:{}/api/", port))
        .json(&payload)
        .send()
        .await
        .unwrap();
    let binding = create_response.json::<serde_json::Value>().await.unwrap();
    let note_id = binding["data"]["note"]["id"].as_str().unwrap();

    let delete_response = client
        .delete(format!("http://localhost:{}/api/{}", port, note_id))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let get_response = client
        .get(format!("http://localhost:{}/api/{}", port, note_id))
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
    teardown_database(&db_pool).await;
}
