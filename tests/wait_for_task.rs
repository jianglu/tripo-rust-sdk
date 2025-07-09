use serde_json::json;
use tripo3d::{TaskState, TripoClient};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CustomResponder;

impl wiremock::Respond for CustomResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        // Use a static counter to simulate state changes
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);

        let (status, progress, result) = if count < 2 {
            // State 1: Running
            ("running", 50, json!({}))
        } else {
            // State 2: Success
            (
                "success",
                100,
                json!({
                    "pbr_model": { "url": "http://example.com/model.glb" }
                }),
            )
        };

        ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "task_id": "mock_task_id_123",
                "type": "text_to_model",
                "status": status,
                "progress": progress,
                "create_time": 123456789,
                "output": null,
                "result": result
            }
        }))
    }
}

#[tokio::test]
async fn test_wait_for_task_with_custom_responder() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/v2/openapi/task/mock_task_id_123"))
        .respond_with(CustomResponder)
        .mount(&server)
        .await;

    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let final_status = client.wait_for_task("mock_task_id_123", true).await.unwrap();

    assert_eq!(final_status.status, TaskState::Success);
    assert!(final_status.result.pbr_model.is_some());
} 