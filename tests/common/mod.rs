use serde_json::json;
use wiremock::matchers::{body_json, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

    // Mock for image_to_3d (simple version)
    Mock::given(method("POST"))
        .and(path("/v2/direct/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "task_id": "mock_task_id_456"
        })))
        .mount(&server)
        .await;

    // Mock for get_task (single successful fetch)
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

    // Mock for model download
    Mock::given(method("GET"))
        .and(path_regex(r"/model_.*\.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes("dummy model data"))
        .mount(&server)
        .await;

    server
}
