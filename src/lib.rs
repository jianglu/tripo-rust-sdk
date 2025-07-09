//! An unofficial Rust SDK for the Tripo3D API.
//!
//! This SDK provides a convenient, asynchronous interface for interacting with the
//! Tripo3D platform to generate 3D models from text or images.

use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

const DEFAULT_API_URL: &str = "https://api.tripo3d.ai/";

/// Represents the possible errors that can occur when using the Tripo SDK.
#[derive(Error, Debug)]
pub enum TripoError {
    /// An error occurred during an HTTP request.
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// An error occurred while parsing a URL.
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    /// The API key was not provided, either directly or via the `TRIPO_API_KEY` environment variable.
    #[error("API key not provided")]
    NoApiKey,
    /// An error returned by the Tripo3D API.
    #[error("API error: {message}")]
    ApiError {
        /// The error message from the API.
        message: String,
    },
}

/// The main client for interacting with the Tripo3D API.
///
/// It holds the HTTP client and the base URL for all API requests.
/// It is cloneable and can be shared across threads.
#[derive(Clone)]
pub struct TripoClient {
    client: reqwest::Client,
    base_url: Url,
}

#[derive(Serialize)]
struct TextTo3DRequest<'a> {
    prompt: &'a str,
    #[serde(rename = "type")]
    type_: &'a str,
}

/// The response from an API call that initiates a task.
#[derive(Deserialize, Debug)]
pub struct TaskResponse {
    /// The unique identifier for the newly created task.
    #[serde(rename = "task_id")]
    pub task_id: String,
}

/// Represents a generated 3D model file.
#[derive(Deserialize, Debug)]
pub struct Model {
    /// The unique identifier for the model.
    pub id: String,
    /// The URL to download the model file.
    pub url: String,
}

/// Represents the state of a generation task.
///
/// This enum is used in `TaskStatus` to provide a clear, typed status,
/// preventing the use of raw strings.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// The task has been received and is waiting to be processed.
    Queued,
    /// The task is actively being processed.
    Processing,
    /// The task completed successfully.
    Success,
    /// The task failed to complete.
    Failed,
    /// The task is in an unknown or unexpected state.
    #[serde(other)]
    Unknown,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TaskState::Queued => "queued",
            TaskState::Processing => "processing",
            TaskState::Success => "success",
            TaskState::Failed => "failed",
            TaskState::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

/// Represents the status of a generation task.
#[derive(Deserialize, Debug)]
pub struct TaskStatus {
    /// The unique identifier for the task.
    pub task_id: String,
    /// The type of the task (e.g., "text_to_model").
    #[serde(rename = "type")]
    pub type_: String,
    /// The current status of the task (e.g., "success", "processing").
    pub status: TaskState,
    /// The progress of the task, from 0 to 100.
    pub progress: u32,
    /// The timestamp when the task was created.
    pub created_at: String,
    /// A list of generated models, available when the task is complete.
    pub models: Option<Vec<Model>>,
}

/// Represents the user's account balance.
#[derive(Deserialize, Debug)]
pub struct Balance {
    /// Total credits granted to the user.
    pub total_granted_credits: f64,
    /// Total credits used by the user.
    pub total_used_credits: f64,
    /// Total credits currently available.
    pub total_available_credits: f64,
}

impl TripoClient {
    /// Creates a new `TripoClient`.
    ///
    /// This method initializes the client with an API key. It first checks for the `api_key`
    /// parameter. If it's `None`, it falls back to the `TRIPO_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns `TripoError::NoApiKey` if the API key is not provided in either way.
    /// Returns `TripoError::RequestError` if the HTTP client fails to build.
    /// Returns `TripoError::UrlError` if the default API URL is invalid.
    pub fn new(api_key: Option<String>) -> Result<Self, TripoError> {
        let api_key = api_key
            .or_else(|| env::var("TRIPO_API_KEY").ok())
            .ok_or(TripoError::NoApiKey)?;
        Self::new_with_url(api_key, DEFAULT_API_URL)
    }

    /// Creates a new `TripoClient` with a custom base URL.
    ///
    /// This is useful for testing or for connecting to a different API endpoint.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication.
    /// * `base_url` - The base URL for the API.
    ///
    /// # Errors
    ///
    /// Returns `TripoError::RequestError` if the HTTP client fails to build.
    /// Returns `TripoError::UrlError` if the provided `base_url` is invalid.
    pub fn new_with_url(api_key: String, base_url: &str) -> Result<Self, TripoError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let base_url = Url::parse(base_url)?;

        Ok(Self { client, base_url })
    }

    /// Submits a new text-to-3D generation task.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A text description of the 3D model to generate.
    ///
    /// # Returns
    ///
    /// A `TaskResponse` containing the ID of the newly created task.
    pub async fn text_to_3d(&self, prompt: &str) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/direct/generate")?;
        let request_body = TextTo3DRequest {
            prompt,
            type_: "text_to_model",
        };

        let response = self.client.post(url).json(&request_body).send().await?;

        if response.status().is_success() {
            let task_response: TaskResponse = response.json().await?;
            Ok(task_response)
        } else {
            let error_response: serde_json::Value = response.json().await?;
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Submits a new image-to-3D generation task.
    ///
    /// # Arguments
    ///
    /// * `image_path` - The path to the image file to use for generation.
    ///
    /// # Returns
    ///
    /// A `TaskResponse` containing the ID of the newly created task.
    pub async fn image_to_3d<P: AsRef<Path>>(
        &self,
        image_path: P,
    ) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/direct/generate")?;

        let file = fs::File::open(image_path)
            .await
            .map_err(|e| TripoError::ApiError {
                message: e.to_string(),
            })?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let some_file = multipart::Part::stream(file_body)
            .file_name("image.png") // You might want to make this dynamic
            .mime_str("image/png")?;

        let form = multipart::Form::new()
            .text("type", "image_to_model")
            .part("file", some_file);

        let response = self.client.post(url).multipart(form).send().await?;

        if response.status().is_success() {
            let task_response: TaskResponse = response.json().await?;
            Ok(task_response)
        } else {
            let error_response: serde_json::Value = response.json().await?;
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Retrieves the status of a specific task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The unique identifier of the task to query.
    ///
    /// # Returns
    ///
    /// A `TaskStatus` struct containing the latest information about the task.
    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatus, TripoError> {
        let url = self
            .base_url
            .join(&format!("v2/organization/tasks/{}", task_id))?;

        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let task_status: TaskStatus = response.json().await?;
            Ok(task_status)
        } else {
            let error_response: serde_json::Value = response.json().await?;
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Retrieves the user's account balance.
    ///
    /// # Returns
    ///
    /// A `Balance` struct containing credit information.
    pub async fn get_balance(&self) -> Result<Balance, TripoError> {
        let url = self.base_url.join("v2/organization/account")?;

        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let balance: Balance = response.json().await?;
            Ok(balance)
        } else {
            let error_response: serde_json::Value = response.json().await?;
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Waits for a task to complete.
    ///
    /// This method polls the `get_task` endpoint until the task status is
    /// either `Success` or `Failed`.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to wait for.
    /// * `verbose` - If `true`, prints the task progress to the console.
    ///
    /// # Returns
    ///
    /// The final `TaskStatus` of the completed or failed task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tripo3d::TripoClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = TripoClient::new(Some("your_api_key".to_string()))?;
    /// let task_id = "some_task_id";
    /// let final_status = client.wait_for_task(task_id, true).await?;
    /// println!("Task finished with status: {}", final_status.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_task(
        &self,
        task_id: &str,
        verbose: bool,
    ) -> Result<TaskStatus, TripoError> {
        loop {
            let task_status = self.get_task(task_id).await?;
            if verbose {
                println!(
                    "Task {}: status={}, progress={}%",
                    task_id, task_status.status, task_status.progress
                );
            }
            match task_status.status {
                TaskState::Success | TaskState::Failed => {
                    return Ok(task_status);
                }
                _ => {
                    // Continue polling
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// Downloads a single model file to a specified directory.
    ///
    /// This function handles the HTTP request to the model's URL and saves the
    /// content to a local file. The filename is inferred from the URL.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the `Model` struct containing the download URL.
    /// * `destination_dir` - The local directory path where the file will be saved.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PathBuf` of the newly created file, or a `TripoError`.
    ///
    /// # Errors
    ///
    /// This function can return an error if the download fails, if the directory
    /// or file cannot be created, or if there's an issue writing the file to disk.
    pub async fn download_model<P: AsRef<Path>>(
        &self,
        model: &Model,
        destination_dir: P,
    ) -> Result<PathBuf, TripoError> {
        let response = self.client.get(&model.url).send().await?;
        if !response.status().is_success() {
            return Err(TripoError::ApiError {
                message: format!("Failed to download file: status {}", response.status()),
            });
        }

        let file_name = model
            .url
            .split('/')
            .last()
            .unwrap_or(&model.id)
            .to_string();
        let dest_path = destination_dir.as_ref().join(file_name);

        fs::create_dir_all(destination_dir.as_ref())
            .await
            .map_err(|e| TripoError::ApiError {
                message: format!("Failed to create directory: {}", e),
            })?;

        let mut file = fs::File::create(&dest_path)
            .await
            .map_err(|e| TripoError::ApiError {
                message: format!("Failed to create file: {}", e),
            })?;

        let content = response.bytes().await?;
        file.write_all(&content).await.map_err(|e| {
            TripoError::ApiError {
                message: format!("Failed to write to file: {}", e),
            }
        })?;

        Ok(dest_path)
    }

    /// Downloads all models from a completed task to a specified directory.
    ///
    /// This is a convenience method that iterates through the models in a `TaskStatus`
    /// and calls `download_model` for each one.
    ///
    /// # Arguments
    ///
    /// * `task` - The completed `TaskStatus` containing the models to download.
    /// * `destination_dir` - The directory where the models will be saved.
    ///
    /// # Returns
    ///
    /// A vector of `PathBuf`s for each downloaded file.
    pub async fn download_all_models<P: AsRef<Path>>(
        &self,
        task: &TaskStatus,
        destination_dir: P,
    ) -> Result<Vec<PathBuf>, TripoError> {
        let mut downloaded_files = Vec::new();
        if let Some(models) = &task.models {
            for model in models {
                let file_path = self
                    .download_model(model, destination_dir.as_ref())
                    .await?;
                downloaded_files.push(file_path);
            }
        }
        Ok(downloaded_files)
    }
} 