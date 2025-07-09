use std::fs;
use tripo3d::{ResultFile, TaskResult, TaskState, TaskStatus, TripoClient};
use wiremock::{
    matchers::{method, path_regex},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_download_model_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"/model_.*\.glb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes("dummy model data"))
        .mount(&server)
        .await;
    
    let client = TripoClient::new_with_url("test_api_key".to_string(), &server.uri()).unwrap();

    let dest_dir = tempfile::tempdir().unwrap();

    let task_status = TaskStatus {
        task_id: "mock_task".to_string(),
        status: TaskState::Success,
        progress: 100,
        create_time: 0,
        output: None,
        result: TaskResult {
            pbr_model: Some(ResultFile {
                url: server.uri() + "/model_download.glb",
            }),
            glb_model: None,
        },
    };

    let downloaded_files = client
        .download_all_models(&task_status, dest_dir.path())
        .await
        .unwrap();

    assert_eq!(downloaded_files.len(), 1);
    let file_path = &downloaded_files[0];
    assert_eq!(
        file_path.file_name().unwrap().to_str().unwrap(),
        "model_download.glb"
    );

    let content = fs::read(file_path).unwrap();
    assert_eq!(content, b"dummy model data");
} 