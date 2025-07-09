use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_json, path_regex};
use serde_json::json;

pub async fn setup_mock_server() -> MockServer {
    let server = MockServer::start().await;

    // Mock for text_to_3d
    Mock::given(method("POST"))
        .and(path("/v2/direct/generate"))
        .and(body_json(json!({
            "prompt": "a delicious hamburger",
            "type": "text_to_model"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "task_id": "mock_task_id_123"
        })))
        .mount(&server)
        .await;

    // Mock for image_to_3d
    Mock::given(method("POST"))
        .and(path("/v2/direct/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "task_id": "mock_task_id_456"
        })))
        .mount(&server)
        .await;

    // Mock for get_task
    Mock::given(method("GET"))
        .and(path("/v2/organization/tasks/mock_task_id_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "task_id": "mock_task_id_123",
            "type": "text_to_model",
            "status": "success",
            "progress": 100,
            "created_at": "2024-01-01T00:00:00Z",
            "models": [
                {
                    "id": "model_id_1",
                    "url": "https://example.com/model1.glb"
                }
            ]
        })))
        .mount(&server)
        .await;
    
    // Mock for get_balance
    Mock::given(method("GET"))
        .and(path("/v2/organization/account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_granted_credits": 1000.0,
            "total_used_credits": 50.0,
            "total_available_credits": 950.0
        })))
        .mount(&server)
        .await;

    server
} 