use tripo3d::TripoClient;
use wiremock::matchers::{method, path, body_json};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

#[tokio::test]
async fn test_text_to_3d_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v2/openapi/task"))
        .and(body_json(json!({
            "prompt": "a delicious hamburger",
            "type": "text_to_model"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "task_id": "mock_task_id_123"
            }
        })))
        .mount(&server)
        .await;
    
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.text_to_3d("a delicious hamburger").await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_123");
} 