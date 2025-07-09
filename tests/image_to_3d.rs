mod common;
use tripo3d::TripoClient;
use common::setup_mock_server;
use std::fs::File;
use std::io::Write;

#[tokio::test]
async fn test_image_to_3d_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    // Create a dummy image file
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test_image.png");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"dummy image data").unwrap();

    let response = client.image_to_3d(file_path).await.unwrap();

    assert_eq!(response.task_id, "mock_task_id_456");
} 