use serde::{Deserialize, Serialize};

/// A private struct for serializing the text-to-3D request body.
#[derive(Serialize)]
pub(crate) struct TextTo3DRequest<'a> {
    pub(crate) prompt: &'a str,
    #[serde(rename = "type")]
    pub(crate) type_: &'a str,
}

/// Represents an object stored in S3.
#[derive(Serialize, Debug)]
pub struct S3Object {
    /// The S3 bucket name.
    pub bucket: String,
    /// The key (path) of the object in the bucket.
    pub key: String,
}

/// Represents the file information sent to the task creation endpoint.
#[derive(Serialize, Debug, Default)]
pub struct FileContent {
    /// The type of file, e.g., "png", "jpg".
    #[serde(rename = "type")]
    pub type_: String,
    /// The S3 object information, if uploaded via S3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<S3Object>,
    /// The URL of the file, if provided directly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The file token, if using a pre-uploaded file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_token: Option<String>,
}

/// The request body for creating an image-to-model task.
#[derive(Serialize, Debug)]
pub struct ImageTaskRequest {
    /// The type of task, e.g., "image_to_model".
    #[serde(rename = "type")]
    pub type_: &'static str,
    /// The file content for the task.
    pub file: FileContent,
}

/// The response from an API call that successfully initiates a task.
#[derive(Deserialize, Debug)]
pub struct TaskResponse {
    /// The unique identifier for the newly created task.
    #[serde(rename = "task_id")]
    pub task_id: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct StsTokenData {
    pub(crate) sts_ak: String,
    pub(crate) sts_sk: String,
    pub(crate) session_token: String,
    pub(crate) resource_bucket: String,
    pub(crate) resource_uri: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct StandardUploadData {
    pub(crate) image_token: String,
}

/// Represents the state of a generation task.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Pending,
    Running,
    Success,
    Failure,
}

/// Represents the file output from a successful task.
#[derive(Debug, Deserialize, Clone)]
pub struct ResultFile {
    pub url: String,
}

/// Represents the different model files in a task result.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TaskResult {
    #[serde(default)]
    pub pbr_model: Option<ResultFile>,
    #[serde(default)]
    pub glb_model: Option<ResultFile>,
}

/// Represents the preview image generated during the task.
#[derive(Debug, Deserialize, Clone)]
pub struct TaskOutput {
    pub generated_image: Option<String>,
}

/// Represents the detailed status and data of a generation task.
#[derive(Debug, Deserialize, Clone)]
pub struct TaskStatus {
    pub task_id: String,
    pub status: TaskState,
    pub progress: u8,
    pub create_time: u64,
    pub result: TaskResult,
    pub output: Option<TaskOutput>,
}

/// Represents the user's account balance and credit information.
#[derive(Deserialize, Debug)]
pub struct Balance {
    /// The available, usable balance.
    pub balance: f64,
    /// The amount of credits that are currently frozen or reserved for ongoing tasks.
    pub frozen: f64,
}

/// A generic wrapper for API responses where the main content is nested under a "data" field.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) data: T,
} 