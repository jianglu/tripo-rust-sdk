mod common;
use std::fs;
use tripo3d::{ResultFile, TaskResult, TaskState, TaskStatus, TripoClient};
use common::setup_mock_server;

#[tokio::test]
async fn test_download_model_success() {
    let server = setup_mock_server().await;
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let dest_dir = tempfile::tempdir().unwrap();
    let task_status = TaskStatus {
        task_id: "mock_task_id_123".to_string(),
        type_: "text_to_model".to_string(),
        status: TaskState::Success,
        progress: 100,
        create_time: 1752091365,
        result: Some(TaskResult {
            pbr_model: ResultFile {
                url: server.uri() + "/model_download.glb",
            },
        }),
    };

    let downloaded_files = client
        .download_all_models(&task_status, dest_dir.path())
        .await
        .unwrap();

    assert_eq!(downloaded_files.len(), 1);
    assert_eq!(
        fs::read_to_string(&downloaded_files[0]).unwrap(),
        "dummy model data"
    );
} 