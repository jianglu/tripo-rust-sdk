use serde_json::json;
use std::sync::{Arc, Mutex};
use tripo3d::{TaskState, TripoClient};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

// A custom responder that changes its response on each call.
struct PollingResponder {
    // Use Arc<Mutex> to safely share state across async calls.
    call_count: Arc<Mutex<u32>>,
}

impl Respond for PollingResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        if *count <= 1 {
            // First call: respond with "processing"
            ResponseTemplate::new(200).set_body_json(json!({
                "task_id": "polling_task_id",
                "type": "text_to_model",
                "status": "processing",
                "progress": 50,
                "created_at": "2024-01-01T00:00:00Z",
                "models": null
            }))
        } else {
            // Subsequent calls: respond with "success"
            ResponseTemplate::new(200).set_body_json(json!({
                "task_id": "polling_task_id",
                "type": "text_to_model",
                "status": "success",
                "progress": 100,
                "created_at": "2024-01-01T00:00:00Z",
                "models": [
                    {
                        "id": "model_id_poll",
                        "url": "https://example.com/model_poll.glb"
                    }
                ]
            }))
        }
    }
}

#[tokio::test]
async fn test_wait_for_task_with_custom_responder() {
    // 1. Set up the mock server
    let server = MockServer::start().await;

    // 2. Create an instance of our custom responder
    let responder = PollingResponder {
        call_count: Arc::new(Mutex::new(0)),
    };

    // 3. Mount the mock with the custom responder
    Mock::given(method("GET"))
        .and(path("/v2/organization/tasks/polling_task_id"))
        .respond_with(responder)
        .mount(&server)
        .await;

    // 4. Set up the client and run the test
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let final_status = client.wait_for_task("polling_task_id", false).await.unwrap();

    // 5. Assert the final status is success
    assert_eq!(final_status.status, TaskState::Success);
    assert_eq!(final_status.progress, 100);
    assert!(final_status.models.is_some());
} 