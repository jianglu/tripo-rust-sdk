mod common;
use tripo3d::TripoClient;
use common::setup_mock_server;

#[tokio::test]
async fn test_get_balance_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.get_balance().await.unwrap();

    assert_eq!(response.total_granted_credits, 1000.0);
    assert_eq!(response.total_used_credits, 50.0);
    assert_eq!(response.total_available_credits, 950.0);
} 