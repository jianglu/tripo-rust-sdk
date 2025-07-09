//! An unofficial Rust SDK for the Tripo3D API.
//!
//! This SDK provides a convenient, asynchronous interface for interacting with the
//! Tripo3D platform to generate 3D models from text prompts or images.
//! It handles API requests, error handling, and file downloads, allowing you to focus on your application's core logic.
//!
//! ## Features
//! - Text-to-3D and Image-to-3D generation.
//! - Asynchronous API for non-blocking operations.
//! - Task polling to wait for generation completion.
//! - Helper functions for downloading generated models.
//! - Typed error handling for robust applications.

use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

const DEFAULT_API_URL: &str = "https://api.tripo3d.ai/";

/// Represents the possible errors that can occur when using the Tripo3D SDK.
#[derive(Error, Debug)]
pub enum TripoError {
    /// An error occurred during an HTTP request (e.g., network issue).
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// An error occurred while parsing a URL, typically the base URL.
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    /// The API key was not provided, either directly or via the `TRIPO_API_KEY` environment variable.
    #[error("API key not provided")]
    NoApiKey,
    /// The API returned a non-successful status code.
    #[error("API error: {message}")]
    ApiError {
        /// The error message returned by the API.
        message: String,
    },
    /// An error occurred during file I/O operations.
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    /// An error occurred during JSON deserialization, indicating a mismatch
    /// between the API response and the expected data structure.
    #[error("JSON deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// The main client for interacting with the Tripo3D API.
///
/// It holds the shared `reqwest::Client` and the base URL for all API requests.
/// It is designed to be cloneable and safe to share across threads.
#[derive(Clone)]
pub struct TripoClient {
    client: reqwest::Client,
    base_url: Url,
}

/// A private struct for serializing the text-to-3D request body.
#[derive(Serialize)]
struct TextTo3DRequest<'a> {
    prompt: &'a str,
    #[serde(rename = "type")]
    type_: &'a str,
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
struct ApiResponse<T> {
    data: T,
}

impl TripoClient {
    /// Creates a new `TripoClient`.
    ///
    /// This method initializes the client with an API key. It first checks for the `api_key`
    /// parameter. If it's `None`, it falls back to the `TRIPO_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// - `TripoError::NoApiKey` if the API key is not provided in either way.
    /// - `TripoError::RequestError` if the internal HTTP client fails to build.
    /// - `TripoError::UrlError` if the default API URL is invalid.
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
    /// * `base_url` - The base URL for the API (e.g., for a mock server).
    ///
    /// # Errors
    ///
    /// - `TripoError::RequestError` if the internal HTTP client fails to build.
    /// - `TripoError::UrlError` if the provided `base_url` is invalid.
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

    /// Submits a new text-to-3D generation task for quick generation.
    ///
    /// This endpoint is designed for fast, direct model generation.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A text description of the 3D model to generate.
    ///
    /// # Returns
    ///
    /// A [`TaskResponse`] containing the ID of the newly created task.
    pub async fn text_to_3d(&self, prompt: &str) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/openapi/task")?;
        let request_body = TextTo3DRequest {
            prompt,
            type_: "text_to_model",
        };

        let response = self.client.post(url).json(&request_body).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<TaskResponse> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error_response: serde_json::Value = response.json().await.unwrap_or_default();
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Submits a new image-to-3D generation task for quick generation.
    ///
    /// This endpoint is designed for fast, direct model generation from an image.
    ///
    /// # Arguments
    ///
    /// * `image_path` - The path to the local image file to use for generation.
    ///
    /// # Returns
    ///
    /// A [`TaskResponse`] containing the ID of the newly created task.
    pub async fn image_to_3d<P: AsRef<Path>>(
        &self,
        image_path: P,
    ) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/openapi/task")?;

        let file = fs::File::open(image_path).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        // TODO: Dynamically determine file name and mime type.
        let some_file = multipart::Part::stream(file_body)
            .file_name("image.png")
            .mime_str("image/png")?;

        let form = multipart::Form::new()
            .text("type", "image_to_model")
            .part("file", some_file);

        let response = self.client.post(url).multipart(form).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<TaskResponse> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error_response: serde_json::Value = response.json().await.unwrap_or_default();
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Retrieves the status of a specific task.
    ///
    /// This is the primary method for polling the status of a long-running generation task.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The unique identifier of the task to query.
    ///
    /// # Returns
    ///
    /// A [`TaskStatus`] struct containing the details of the task.
    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatus, TripoError> {
        let url = self
            .base_url
            .join(&format!("v2/openapi/task/{}", task_id))?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<TaskStatus> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error_response: serde_json::Value = response.json().await.unwrap_or_default();
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Retrieves the user's account balance.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the [`Balance`] on success, or a [`TripoError`] on failure.
    pub async fn get_balance(&self) -> Result<Balance, TripoError> {
        let url = self.base_url.join("v2/openapi/user/balance")?;
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<Balance> = response.json().await?;
            Ok(api_response.data)
        } else {
            let error_body: serde_json::Value = response.json().await.unwrap_or_default();
            Err(TripoError::ApiError {
                message: format!("API error: {}", error_body),
            })
        }
    }

    /// Waits for a task to complete by polling its status.
    ///
    /// This method repeatedly calls `get_task` until the task status is
    /// either `Success` or `Failed`.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to wait for.
    /// * `verbose` - If `true`, prints the task progress to the console.
    ///
    /// # Returns
    ///
    /// The final [`TaskStatus`] of the completed or failed task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tripo3d::TripoClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = TripoClient::new(Some("your_api_key".to_string()))?;
    /// # let task_id = "some_task_id";
    /// let final_status = client.wait_for_task(task_id, true).await?;
    /// println!("Task finished with status: {:?}", final_status.status);
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
                    "Task status: {:?}, progress: {}%",
                    task_status.status, task_status.progress
                );
            }
            match task_status.status {
                TaskState::Success | TaskState::Failure => {
                    return Ok(task_status);
                }
                _ => {
                    // Continue polling after a short delay.
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
    /// * `model` - A reference to a [`Model`] struct containing the download URL.
    /// * `destination_dir` - The local directory path where the file will be saved.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PathBuf` of the newly created file, or a [`TripoError`].
    ///
    /// # Errors
    ///
    /// This function can return an error if the download fails, if the destination
    /// directory or file cannot be created, or if there's an issue writing the file to disk.
    pub async fn download_model<P: AsRef<Path>>(
        &self,
        model_file: &ResultFile,
        dest_dir: P,
    ) -> Result<PathBuf, TripoError> {
        let parsed_url = Url::parse(&model_file.url)?;
        let file_name = parsed_url
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("downloaded_model.bin");

        let file_path = dest_dir.as_ref().join(file_name);
        let response = self.client.get(model_file.url.clone()).send().await?;

        if !response.status().is_success() {
            return Err(TripoError::ApiError {
                message: format!("Failed to download file: status {}", response.status()),
            });
        }

        fs::create_dir_all(dest_dir.as_ref()).await?;

        let mut file = fs::File::create(&file_path).await?;
        let content = response.bytes().await?;
        file.write_all(&content).await?;

        Ok(file_path)
    }

    /// Downloads all models from a completed task to a specified directory.
    ///
    /// This is a convenience method that extracts the model URL from a [`TaskStatus`]
    /// and calls `download_model`.
    ///
    /// # Arguments
    ///
    /// * `task` - The completed [`TaskStatus`] containing the model to download.
    /// * `destination_dir` - The directory where the model will be saved.
    ///
    /// # Returns
    ///
    /// A `Vec` containing the `PathBuf` of the downloaded file. The vector will
    /// be empty if the task has no result.
    pub async fn download_all_models<P: AsRef<Path>>(
        &self,
        task_status: &TaskStatus,
        dest_dir: P,
    ) -> Result<Vec<PathBuf>, TripoError> {
        let mut downloaded_files = Vec::new();

        if let Some(pbr_model) = &task_status.result.pbr_model {
            let file_path = self.download_model(pbr_model, &dest_dir).await?;
            downloaded_files.push(file_path);
        }

        if let Some(glb_model) = &task_status.result.glb_model {
            let file_path = self.download_model(glb_model, &dest_dir).await?;
            downloaded_files.push(file_path);
        }

        Ok(downloaded_files)
    }
} 