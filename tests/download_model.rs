mod common;
use tripo3d::{Model, TaskState, TaskStatus, TripoClient};
use common::setup_mock_server;
use std::fs;

#[tokio::test]
async fn test_download_model_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let model_url = format!("{}/model_poll.glb", server.uri());
    let task = TaskStatus {
        task_id: "polling_task_id".to_string(),
        type_: "text_to_model".to_string(),
        status: TaskState::Success,
        progress: 100,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        models: Some(vec![Model {
            id: "model_id_poll".to_string(),
            url: model_url,
        }]),
    };

    let dir = tempfile::tempdir().unwrap();
    let downloaded_files = client.download_all_models(&task, dir.path()).await.unwrap();

    assert_eq!(downloaded_files.len(), 1);
    let file_path = &downloaded_files[0];
    assert_eq!(file_path.file_name().unwrap(), "model_poll.glb");
    
    let content = fs::read_to_string(file_path).unwrap();
    assert_eq!(content, "dummy model data");
} 