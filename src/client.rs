use crate::error::TripoError;
use crate::types::{
    ApiResponse, Balance, ResultFile, TaskResponse, TaskState, TaskStatus, TextTo3DRequest,
};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::multipart;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

const DEFAULT_API_URL: &str = "https://api.tripo3d.ai/";

/// The main client for interacting with the Tripo3D API.
///
/// It holds the shared `reqwest::Client` and the base URL for all API requests.
/// It is designed to be cloneable and safe to share across threads.
#[derive(Clone)]
pub struct TripoClient {
    client: reqwest::Client,
    base_url: Url,
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
        let image_path = image_path.as_ref();
        let url = self.base_url.join("v2/openapi/task")?;

        let file = fs::File::open(image_path).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let file_name = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.bin")
            .to_string();

        let mime_type = mime_guess::from_path(image_path)
            .first_or_octet_stream()
            .to_string();

        #[derive(serde::Serialize)]
        struct ImageRequest<'a> {
            #[serde(rename = "type")]
            type_: &'a str,
        }
        let request_data = ImageRequest {
            type_: "image_to_model",
        };
        let json_part = multipart::Part::text(serde_json::to_string(&request_data)?)
            .mime_str("application/json")?;

        let file_part = multipart::Part::stream(file_body)
            .file_name(file_name)
            .mime_str(&mime_type)?;

        let form = multipart::Form::new()
            .part("json", json_part)
            .part("file", file_part);

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