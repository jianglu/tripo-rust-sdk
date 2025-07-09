use serde::{Deserialize, Serialize};

/// A private struct for serializing the text-to-3D request body.
#[derive(Serialize)]
pub(crate) struct TextTo3DRequest<'a> {
    pub(crate) prompt: &'a str,
    #[serde(rename = "type")]
    pub(crate) type_: &'a str,
}

/// The response from an API call that successfully initiates a task.
#[derive(Deserialize, Debug)]
pub struct TaskResponse {
    /// The unique identifier for the newly created task.
    #[serde(rename = "task_id")]
    pub task_id: String,
}

/// A temporary struct used to facilitate model downloading.
///
/// This struct is created internally by `download_all_models` to pass
/// the necessary URL and a placeholder ID to the `download_model` function.
#[derive(Deserialize, Debug)]
pub struct Model {
    /// The unique identifier for the model.
    pub id: String,
    /// The URL to download the model file.
    pub url: String,
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