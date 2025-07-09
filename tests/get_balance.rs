use tripo3d::TripoClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

#[tokio::test]
async fn test_get_balance_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/openapi/user/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "balance": 950.0,
                "frozen": 50.0
            }
        })))
        .mount(&server)
        .await;
    
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let response = client.get_balance().await.unwrap();

    assert_eq!(response.balance, 950.0);
    assert_eq!(response.frozen, 50.0);
} 