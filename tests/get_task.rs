mod common;
use tripo3d::{TripoClient, TaskState};
use common::setup_mock_server;

#[tokio::test]
async fn test_get_task_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.get_task("mock_task_id_123").await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_123");
    assert_eq!(response.status, TaskState::Success);
    assert!(response.result.is_some());
    let result = response.result.unwrap();
    assert_eq!(result.pbr_model.url, "https://example.com/model1.glb");
} 