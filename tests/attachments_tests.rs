use axum::Router;
use reqwest::{multipart, Client, StatusCode};
use serde_json::{json, Value};
use std::{env, sync::Arc};
use tokio::fs;
use uuid::Uuid;

use rust_axum_postgres_api::{
    api::{attachments::router::create_attachments_routes, notes::router::create_notes_routes},
    infrastructure::database::AppState,
};

mod common;
use common::{setup_database, start_test_server, teardown_database};

const TEST_UPLOAD_DIR: &str = "uploads/";

async fn setup_test_environment(test_name: &str) -> (Client, u16, sqlx::PgPool, Uuid) {
    tokio::fs::create_dir_all(TEST_UPLOAD_DIR)
        .await
        .expect("Failed to create uploads directory");

    let db_pool = setup_database().await;
    let app_state = Arc::new(AppState {
        db: db_pool.clone(),
    });

    let app = Router::new()
        .merge(create_notes_routes(app_state.clone()))
        .merge(create_attachments_routes(app_state));

    let port = start_test_server(app).await;
    let client = Client::new();

    let note_payload = json!({
        "title": format!("Test Note for {} - {}", test_name, Uuid::new_v4()),
        "content": "Content for testing attachments",
        "category": "test",
        "published": true,
        "tags": []
    });

    let note_response = client
        .post(format!("http://localhost:{}/api/", port))
        .json(&note_payload)
        .send()
        .await
        .expect("Failed to create test note");

    assert_eq!(
        note_response.status(),
        StatusCode::CREATED,
        "Failed to create note. Status: {}. Response: {}",
        note_response.status(),
        note_response.text().await.unwrap_or_default()
    );

    let response_json = note_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse note response");

    let note_id = response_json["data"]["note"]["id"]
        .as_str()
        .expect("Failed to get note ID")
        .parse::<Uuid>()
        .expect("Failed to parse note ID as UUID");

    (client, port, db_pool, note_id)
}

async fn cleanup_test_environment(db_pool: &sqlx::PgPool) {
    teardown_database(db_pool).await;
    let _ = fs::remove_dir_all(TEST_UPLOAD_DIR).await;
}

#[tokio::test]
async fn test_upload_attachment() {
    let (client, port, db_pool, note_id) = setup_test_environment("upload_attachment").await;

    let file_content = "Test file content";
    let file_name = "test.txt";
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_content.as_bytes().to_vec())
            .file_name(file_name.to_string())
            .mime_str("text/plain")
            .expect("Failed to set mime type"),
    );

    let response = client
        .post(format!(
            "http://localhost:{}/api/notes/{}/attachments",
            port, note_id
        ))
        .multipart(form)
        .send()
        .await
        .expect("Failed to send upload request");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Upload failed. Status: {}. Response: {}",
        response.status(),
        response.text().await.unwrap_or_default()
    );

    cleanup_test_environment(&db_pool).await;
}

#[tokio::test]
async fn test_list_attachments() {
    let (client, port, db_pool, note_id) = setup_test_environment("list_attachments").await;

    let response = client
        .get(format!(
            "http://localhost:{}/api/notes/{}/attachments",
            port, note_id
        ))
        .send()
        .await
        .expect("Failed to list attachments");

    assert_eq!(response.status(), StatusCode::OK);

    cleanup_test_environment(&db_pool).await;
}

#[tokio::test]
async fn test_download_attachment() {
    let (client, port, db_pool, note_id) = setup_test_environment("download_attachment").await;

    // Ensure upload directory exists with proper permissions
    tokio::fs::create_dir_all(TEST_UPLOAD_DIR)
        .await
        .expect("Failed to create upload directory");

    let file_content = "Test file content";
    let file_name = "test.txt";

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_content.as_bytes().to_vec())
            .file_name(file_name.to_string())
            .mime_str("text/plain")
            .expect("Failed to set mime type"),
    );

    // Upload the attachment
    let upload_response = client
        .post(format!(
            "http://localhost:{}/api/notes/{}/attachments",
            port, note_id
        ))
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload attachment");

    assert_eq!(upload_response.status(), StatusCode::OK);

    let response_text = upload_response
        .text()
        .await
        .expect("Failed to get response text");
    let attachment_data: serde_json::Value =
        serde_json::from_str(&response_text).expect("Failed to parse response JSON");

    // Extract attachment details and ensure they're valid
    let attachment_id = Uuid::parse_str(
        attachment_data["data"]["attachment"]["id"]
            .as_str()
            .expect("Failed to get attachment ID"),
    )
    .expect("Invalid UUID format");

    let file_path = attachment_data["data"]["attachment"]["file_path"]
        .as_str()
        .expect("Failed to get file path");

    // Verify file exists and wait for filesystem operations to complete
    for _ in 0..3 {
        if tokio::fs::metadata(file_path).await.is_ok() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    assert!(
        tokio::fs::metadata(file_path).await.is_ok(),
        "Uploaded file does not exist at path: {}",
        file_path
    );

    // Download the attachment
    let download_response = client
        .get(format!(
            "http://localhost:{}/api/attachments/{}/download",
            port, attachment_id
        ))
        .send()
        .await
        .expect("Failed to download attachment");

    assert_eq!(download_response.status(), StatusCode::OK);

    let downloaded_content = download_response
        .bytes()
        .await
        .expect("Failed to get download content");

    assert_eq!(
        std::str::from_utf8(&downloaded_content).unwrap(),
        file_content,
        "Downloaded content does not match uploaded content"
    );

    cleanup_test_environment(&db_pool).await;
}

#[tokio::test]
async fn test_delete_attachment() {
    let (client, port, db_pool, note_id) = setup_test_environment("delete_attachment").await;

    let file_content = "Test file content";
    let file_name = "test.txt";

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(file_content.as_bytes().to_vec())
            .file_name(file_name.to_string())
            .mime_str("text/plain")
            .expect("Failed to set mime type"),
    );

    // Upload the attachment.
    let upload_url = format!(
        "http://localhost:{}/api/notes/{}/attachments",
        port, note_id
    );
    let upload_response = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .expect("Failed to upload attachment");

    assert_eq!(
        upload_response.status(),
        StatusCode::OK,
        "Upload failed. Status: {}. Response: {}",
        upload_response.status(),
        upload_response.text().await.unwrap_or_default()
    );

    let response_text = upload_response
        .text()
        .await
        .expect("Failed to get response text");
    println!("Upload Response: {}", response_text);

    let attachment_data: Value =
        serde_json::from_str(&response_text).expect("Failed to parse response JSON");

    let attachment_id = Uuid::parse_str(
        attachment_data["data"]["attachment"]["id"]
            .as_str()
            .expect("Failed to get attachment ID"),
    )
    .expect("Invalid UUID format");

    let file_path_str = attachment_data["data"]["attachment"]["file_path"]
        .as_str()
        .expect("Failed to get file path");
    println!("File path from response: {}", file_path_str);
    println!(
        "Current working directory: {:?}",
        env::current_dir().unwrap()
    );

    // Convert the relative file path to an absolute path.
    let cwd = env::current_dir().unwrap();
    let absolute_file_path = cwd.join(file_path_str);

    // Loop for up to 1 second to give file operations a chance to complete.
    let mut file_exists = false;
    for _ in 0..10 {
        if fs::metadata(&absolute_file_path).await.is_ok() {
            file_exists = true;
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    assert!(
        file_exists,
        "File was not created on filesystem at '{:?}'",
        absolute_file_path
    );

    // Verify that the attachment is accessible via API.
    let verify_url = format!(
        "http://localhost:{}/api/attachments/{}/download",
        port, attachment_id
    );
    let verify_response = client
        .get(&verify_url)
        .send()
        .await
        .expect("Failed to verify attachment");
    let verify_status = verify_response.status();
    let verify_text = verify_response
        .text()
        .await
        .expect("Failed to get verify response text");
    println!("Verify Status: {}", verify_status);
    println!("Verify Response: {}", verify_text);

    assert_eq!(
        verify_status,
        StatusCode::OK,
        "Attachment verification failed. Status: {}. Response: {}",
        verify_status,
        verify_text
    );

    // Delete the attachment.
    let delete_url = format!(
        "http://localhost:{}/api/attachments/{}",
        port, attachment_id
    );
    let delete_response = client
        .delete(&delete_url)
        .send()
        .await
        .expect("Failed to send delete request");
    assert_eq!(
        delete_response.status(),
        StatusCode::NO_CONTENT,
        "Delete failed. Status: {}. Response: {}",
        delete_response.status(),
        delete_response.text().await.unwrap_or_default()
    );

    // Wait briefly for the deletion to propagate.
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify deletion by attempting to download the attachment.
    let verify_deleted_response = client
        .get(&verify_url)
        .send()
        .await
        .expect("Failed to verify deletion");
    assert_eq!(
        verify_deleted_response.status(),
        StatusCode::NOT_FOUND,
        "Attachment still accessible after deletion"
    );

    cleanup_test_environment(&db_pool).await;
}
