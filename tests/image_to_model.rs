use tripo3d::TripoClient;
use wiremock::matchers::{method, path, body_json};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;
use std::fs::File;
use std::io::Write;

// --- Test Case 1: Uploading a local file ---
#[tokio::test]
async fn test_image_to_model_with_local_file() {
    let server = MockServer::start().await;

    // Mock file upload endpoint (multipart)
    let file_token = "mock-file-token-from-upload";
    Mock::given(method("POST"))
        .and(path("upload/sts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "image_token": file_token
            }
        })))
        .mount(&server)
        .await;

    // Mock task creation endpoint, expecting file_token
    let expected_task_body = json!({
        "type": "image_to_model",
        "file": { "type": "png", "file_token": file_token }
    });
    Mock::given(method("POST"))
        .and(path("task"))
        .and(body_json(expected_task_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "task_id": "task_from_file" }
        })))
        .mount(&server)
        .await;

    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.png");
    File::create(&file_path).unwrap().write_all(b"dummy").unwrap();

    let response = client.image_to_model(file_path.to_str().unwrap()).await.unwrap();
    assert_eq!(response.task_id, "task_from_file");
}

// --- Test Case 2: Using a URL ---
#[tokio::test]
async fn test_image_to_model_with_url() {
    let server = MockServer::start().await;
    let image_url = "http://example.com/image.jpeg";

    // Mock task creation endpoint, expecting a URL
    let expected_task_body = json!({
        "type": "image_to_model",
        "file": { "type": "jpeg", "url": image_url }
    });
    Mock::given(method("POST"))
        .and(path("task"))
        .and(body_json(expected_task_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "task_id": "task_from_url" }
        })))
        .mount(&server)
        .await;

    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let response = client.image_to_model(image_url).await.unwrap();
    assert_eq!(response.task_id, "task_from_url");
}

// --- Test Case 3: Using a File Token ---
#[tokio::test]
async fn test_image_to_model_with_file_token() {
    let server = MockServer::start().await;
    let file_token = "123e4567-e89b-12d3-a456-426614174000";

    // Mock task creation endpoint, expecting a file token
    let expected_task_body = json!({
        "type": "image_to_model",
        "file": { "type": "jpeg", "file_token": file_token }
    });
    Mock::given(method("POST"))
        .and(path("task"))
        .and(body_json(expected_task_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "task_id": "task_from_token" }
        })))
        .mount(&server)
        .await;

    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let response = client.image_to_model(file_token).await.unwrap();
    assert_eq!(response.task_id, "task_from_token");
} 