use serde::{Deserialize, Serialize};

/// A private struct for serializing the text-to-model request body.
#[derive(Serialize)]
pub(crate) struct TextToModelRequest<'a> {
    pub(crate) prompt: &'a str,
    #[serde(rename = "type")]
    pub(crate) type_: &'a str,
}

/// Represents an object stored in an S3-compatible service.
#[derive(Serialize, Debug)]
pub struct S3Object {
    /// The name of the S3 bucket.
    pub bucket: String,
    /// The full key (path) of the object within the bucket.
    pub key: String,
}

/// Describes the input file for a generation task.
///
/// This struct is flexible and can represent a file in one of three ways:
/// 1. As an object in an S3 bucket (`object`).
/// 2. As a publicly accessible URL (`url`).
/// 3. As a token representing a previously uploaded file (`file_token`).
#[derive(Serialize, Debug, Default)]
pub struct FileContent {
    /// The file format, e.g., "png", "jpeg".
    #[serde(rename = "type")]
    pub type_: String,
    /// The S3 object details, if the file was uploaded via STS tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<S3Object>,
    /// A direct URL to the image file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A token representing a file uploaded via the standard multipart endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_token: Option<String>,
}

/// A request to create an image-to-model task.
#[derive(Serialize, Debug)]
pub struct ImageTaskRequest {
    /// The type of the task, which should be "image_to_model".
    #[serde(rename = "type")]
    pub type_: &'static str,
    /// The file content to be used for the task.
    pub file: FileContent,
}

/// The response from an API call that successfully initiates a task.
#[derive(Deserialize, Debug)]
pub struct TaskResponse {
    /// The unique identifier for the newly created task.
    #[serde(rename = "task_id")]
    pub task_id: String,
}

/// (Internal) Holds temporary STS credentials for uploading to S3.
#[derive(Deserialize, Debug)]
pub(crate) struct StsTokenData {
    pub(crate) sts_ak: String,
    pub(crate) sts_sk: String,
    pub(crate) session_token: String,
    pub(crate) resource_bucket: String,
    pub(crate) resource_uri: String,
}

/// (Internal) Holds the file token from a standard multipart upload.
#[derive(Deserialize, Debug)]
pub(crate) struct StandardUploadData {
    pub(crate) image_token: String,
}

/// Represents the lifecycle state of a generation task.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// The task has been submitted but has not yet started processing.
    Pending,
    /// The task is actively being processed.
    Running,
    /// The task completed successfully.
    Success,
    /// The task failed to complete.
    Failure,
}

/// A downloadable file asset, typically a 3D model.
#[derive(Debug, Deserialize, Clone)]
pub struct ResultFile {
    /// The direct URL to download the file.
    pub url: String,
}

/// The set of output files from a successfully completed task.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TaskResult {
    /// The primary model output in PBR (Physically-Based Rendering) format, typically GLB.
    #[serde(default)]
    pub pbr_model: Option<ResultFile>,
    /// An alternative model output in GLB format.
    #[serde(default)]
    pub glb_model: Option<ResultFile>,
}

/// A preview image generated during the task.
#[derive(Debug, Deserialize, Clone)]
pub struct TaskOutput {
    /// The URL of the generated preview image.
    pub generated_image: Option<String>,
}

/// The detailed status and data of a generation task.
#[derive(Debug, Deserialize, Clone)]
pub struct TaskStatus {
    /// The unique identifier of the task.
    pub task_id: String,
    /// The current lifecycle state of the task.
    pub status: TaskState,
    /// The completion progress of the task, from 0 to 100.
    pub progress: u8,
    /// The Unix timestamp of when the task was created.
    pub create_time: u64,
    /// The resulting output files from the task, if successful.
    pub result: TaskResult,
    /// A link to a generated preview image, if available.
    pub output: Option<TaskOutput>,
}

/// The user's account balance.
#[derive(Deserialize, Debug)]
pub struct Balance {
    /// The available, usable balance.
    pub balance: f64,
    /// The amount of credits currently reserved for ongoing tasks.
    pub frozen: f64,
}

/// (Internal) A generic wrapper for API responses where the content is nested under a "data" field.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) data: T,
} 