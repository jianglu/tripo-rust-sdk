mod common;
use tripo3d::TripoClient;
use common::setup_mock_server;

#[tokio::test]
async fn test_get_task_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.get_task("mock_task_id_123").await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_123");
    assert_eq!(response.status, "success");
    assert_eq!(response.progress, 100);
    assert!(response.models.is_some());
    let models = response.models.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model_id_1");
} 