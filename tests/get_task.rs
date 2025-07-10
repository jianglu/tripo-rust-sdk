use tripo3d::{TaskState, TaskStatus, TripoClient};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};
use serde_json::json;

#[tokio::test]
async fn test_get_task_success() {
    let server = MockServer::start().await;
    let task_id = "mock_task_id_123";

    Mock::given(method("GET"))
        .and(path(&format!("task/{}", task_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "task_id": "mock_task_id_123",
                "type": "text_to_model",
                "status": "success",
                "progress": 100,
                "create_time": 1752091365,
                "output": {
                    "generated_image": "https://example.com/image.webp"
                },
                "result": {
                    "pbr_model": {
                        "url": "https://example.com/model1.glb"
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();
    let response: TaskStatus = client.get_task(task_id).await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_123");
    assert_eq!(response.status, TaskState::Success);
    assert!(response.result.pbr_model.is_some());
    let pbr_model = response.result.pbr_model.unwrap();
    assert_eq!(pbr_model.url, "https://example.com/model1.glb");
} 