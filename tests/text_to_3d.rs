mod common;
use tripo3d::TripoClient;
use common::setup_mock_server;

#[tokio::test]
async fn test_text_to_3d_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.text_to_3d("a delicious hamburger").await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_123");
} 