use crate::error::TripoError;
use crate::types::{
    ApiResponse, Balance, FileContent, ImageTaskRequest, ResultFile, S3Object, StandardUploadData,
    StsTokenData, TaskResponse, TaskState, TaskStatus, TextToModelRequest,
};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use url::Url;

use aws_credential_types::Credentials;
use aws_sdk_s3::config::SharedCredentialsProvider;
use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use futures_util::{Stream, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::multipart;
use tokio::fs::File;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::codec::{BytesCodec, FramedRead};

const DEFAULT_API_URL: &str = "https://api.tripo3d.ai/v2/openapi/";

static UUID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

/// The main client for interacting with the Tripo3D API.
///
/// It holds the shared `reqwest::Client` and the base URL for all API requests.
/// It is designed to be cloneable and safe to share across threads.
#[derive(Clone)]
pub struct TripoClient {
    client: reqwest::Client,
    base_url: Url,
    api_key: String,
    /// (For testing) Overrides the S3 endpoint to allow mocking S3 uploads.
    pub s3_endpoint_override: Option<String>,
}

impl TripoClient {
    /// Creates a new `TripoClient`.
    ///
    /// This method initializes the client with an API key. It first checks for the `api_key`
    /// parameter. If it's `None`, it falls back to the `TRIPO_API_KEY` environment variable.
    ///
    /// # Arguments
    ///
    /// * `api_key` - An `Option<String>` containing the API key.
    ///
    /// # Errors
    ///
    /// Returns `TripoError::MissingApiKey` if the API key is not provided either via the parameter or the environment variable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use tripo3d::TripoClient;
    /// // Create a client using a provided API key
    /// let client_from_key = TripoClient::new(Some("your_api_key_here".to_string()));
    ///
    /// // Or create a client using the TRIPO_API_KEY environment variable
    /// // (ensure it's set in your environment)
    /// let client_from_env = TripoClient::new(None);
    /// ```
    pub fn new(api_key: Option<String>) -> Result<Self, TripoError> {
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
    /// This function can return an error if the internal HTTP client fails to build or if the provided `base_url` is invalid.
    pub fn new_with_url(api_key: Option<String>, base_url: &str) -> Result<Self, TripoError> {
        let api_key = api_key.or_else(|| env::var("TRIPO_API_KEY").ok());
        let Some(api_key) = api_key else {
            return Err(TripoError::MissingApiKey);
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let base_url = Url::parse(base_url)?;

        Ok(Self {
            client,
            base_url,
            api_key,
            s3_endpoint_override: None,
        })
    }

    /// Submits a new text-to-model generation task.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A text description of the 3D model to generate.
    ///
    /// # Returns
    ///
    /// On success, a [`TaskResponse`] containing the ID of the newly created task.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the API request fails.
    pub async fn text_to_model(&self, prompt: &str) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("task")?;
        let request_body = TextToModelRequest {
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

    /// Uploads a file to a temporary S3 location using STS credentials.
    ///
    /// This method replicates a secondary upload mechanism from the official Python SDK.
    /// It first requests temporary STS credentials from the Tripo API, then uses those
    /// credentials to upload the specified file directly to an S3 bucket.
    ///
    /// **Note**: This is generally not the primary method for file uploads.
    /// `upload_file` is preferred for most use cases.
    ///
    /// # Arguments
    ///
    /// * `image_path` - The path to the local image file to upload.
    ///
    /// # Returns
    ///
    /// On success, a [`FileContent`] struct containing the S3 object details.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if fetching STS tokens, reading the file, or uploading to S3 fails.
    pub async fn upload_file_s3<P: AsRef<Path>>(
        &self,
        image_path: P,
    ) -> Result<FileContent, TripoError> {
        // 1. Get STS token from Tripo API
        let url = self.base_url.join("upload/sts/token")?;
        let sts_response: ApiResponse<StsTokenData> = self
            .client
            .post(url)
            .json(&serde_json::json!({ "format": "jpeg" }))
            .send()
            .await?
            .json()
            .await?;
        let sts_data = sts_response.data;

        // 2. Configure S3 client with the temporary credentials
        let s3_credentials = Credentials::new(
            sts_data.sts_ak.clone(),
            sts_data.sts_sk.clone(),
            Some(sts_data.session_token.clone()),
            None, // No expiration time needed here
            "TripoStsProvider",
        );

        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&aws_config)
            .credentials_provider(SharedCredentialsProvider::new(s3_credentials));

        if let Some(endpoint_url) = &self.s3_endpoint_override {
            s3_config_builder = s3_config_builder
                .region(aws_sdk_s3::config::Region::new("us-east-1"))
                .endpoint_url(endpoint_url)
                .force_path_style(true);
        }

        let s3_config = s3_config_builder.build();
        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

        // 3. Upload file to S3
        let body = ByteStream::from_path(image_path.as_ref()).await?;

        s3_client
            .put_object()
            .bucket(sts_data.resource_bucket.clone())
            .key(sts_data.resource_uri.clone())
            .body(body)
            .send()
            .await
            .map_err(|e| TripoError::ApiError {
                message: format!("S3 upload failed: {}", e),
            })?;

        // 4. Return the file content structure
        let s3_object = S3Object {
            bucket: sts_data.resource_bucket,
            key: sts_data.resource_uri,
        };

        let extension = image_path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpeg")
            .to_string();

        Ok(FileContent {
            type_: extension,
            object: Some(s3_object),
            ..Default::default()
        })
    }

    /// Uploads a file using the standard multipart method to get a file token.
    ///
    /// This is the primary and recommended method for uploading files. It sends the file
    /// directly to the Tripo API as a `multipart/form-data` request and receives a `file_token`
    /// in return, which can then be used in other API calls.
    ///
    /// # Arguments
    ///
    /// * `image_path` - The path to the local image file to upload.
    ///
    /// # Returns
    ///
    /// On success, a `file_token` as a `String`.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the file cannot be read or if the API request fails.
    pub async fn upload_file<P: AsRef<Path>>(&self, image_path: P) -> Result<String, TripoError> {
        let image_path = image_path.as_ref();
        let url = self.base_url.join("upload/sts")?;

        let file = File::open(image_path).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let file_name = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                TripoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Could not determine file name",
                ))
            })?
            .to_string();

        let mime_type = mime_guess::from_path(image_path)
            .first_or_octet_stream()
            .to_string();

        let file_part = multipart::Part::stream(file_body)
            .file_name(file_name)
            .mime_str(&mime_type)?;

        let form = multipart::Form::new().part("file", file_part);

        let response = self.client.post(url).multipart(form).send().await?;

        if response.status().is_success() {
            let api_response: ApiResponse<StandardUploadData> = response.json().await?;
            Ok(api_response.data.image_token)
        } else {
            let error_response: serde_json::Value = response.json().await.unwrap_or_default();
            Err(TripoError::ApiError {
                message: error_response.to_string(),
            })
        }
    }

    /// Submits a new image-to-model generation task.
    ///
    /// The `image` parameter is flexible and accepts one of three input types:
    /// 1. A public URL string starting with `http://` or `https://`.
    /// 2. A file token (as a UUID string) obtained from a previous upload.
    /// 3. A path to a local file, which will be uploaded automatically.
    ///
    /// # Arguments
    ///
    /// * `image` - A string representing the image input (URL, file token, or local path).
    ///
    /// # Returns
    ///
    /// On success, a [`TaskResponse`] containing the ID of the newly created task.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the input string is a file path that doesn't exist,
    /// if the file upload fails, or if the final API request fails.
    pub async fn image_to_model(&self, image: &str) -> Result<TaskResponse, TripoError> {
        let file_content = self._create_file_content_from_str(image).await?;

        let request_body = ImageTaskRequest {
            type_: "image_to_model",
            file: file_content,
        };

        let url = self.base_url.join("task")?;
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

    async fn _create_file_content_from_str(
        &self,
        image_str: &str,
    ) -> Result<FileContent, TripoError> {
        let file_content;

        if image_str.starts_with("http://") || image_str.starts_with("https://") {
            file_content = FileContent {
                url: Some(image_str.to_string()),
                type_: "jpeg".to_string(),
                ..Default::default()
            };
        } else if UUID_RE.is_match(image_str) {
            file_content = FileContent {
                file_token: Some(image_str.to_string()),
                type_: "jpeg".to_string(),
                ..Default::default()
            };
        } else {
            let path = Path::new(image_str);
            if !path.exists() {
                return Err(TripoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Image file not found: {}", image_str),
                )));
            }
            // If it's a local file, upload it via multipart and get a file_token
            let file_token = self.upload_file(path).await?;
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpeg")
                .to_string();

            file_content = FileContent {
                file_token: Some(file_token),
                type_: extension,
                ..Default::default()
            };
        }

        Ok(file_content)
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
    /// On success, a [`TaskStatus`] struct with the latest status of the task.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the API request fails.
    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatus, TripoError> {
        let url = self.base_url.join(&format!("task/{}", task_id))?;
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

    /// Watches a single task for real-time status updates using WebSockets.
    ///
    /// This is a more efficient alternative to polling `get_task`. It opens a WebSocket
    /// connection and yields `TaskStatus` updates as they are received from the server.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The ID of the task to watch.
    ///
    /// # Returns
    ///
    /// On success, a `Stream` that yields `Result<TaskStatus, TripoError>` items.
    /// The stream closes when the server closes the connection (typically after the task completes).
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the initial WebSocket connection fails. Stream items can be errors
    /// if a message is received that cannot be parsed or if a transport error occurs.
    pub async fn watch_task(
        &self,
        task_id: &str,
    ) -> Result<impl Stream<Item = Result<TaskStatus, TripoError>>, TripoError> {
        let ws_base_url = self.get_ws_base_url()?;
        let watch_url = ws_base_url.join(&format!("task/watch/{}", task_id))?;
        self.connect_and_stream_tasks(watch_url).await
    }

    /// Watches all tasks for real-time status updates using WebSockets.
    ///
    /// It opens a WebSocket connection and yields `TaskStatus` updates as they are received.
    /// An optional timestamp can be provided to receive updates since that time.
    ///
    /// # Arguments
    ///
    /// * `since` - An optional `DateTime<Utc>` to get updates from a specific point in time.
    ///             If `None`, it starts watching for new updates from the present moment.
    ///
    /// # Returns
    ///
    /// A `Stream` that yields `Result<TaskStatus, TripoError>` items.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the initial connection fails.
    pub async fn watch_all_tasks(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> Result<impl Stream<Item = Result<TaskStatus, TripoError>>, TripoError> {
        let ws_base_url = self.get_ws_base_url()?;
        let watch_url = if let Some(time) = since {
            ws_base_url.join(&format!("task/watch/all/{}", time.to_rfc3339()))?
        } else {
            ws_base_url.join("task/watch/all")?
        };
        self.connect_and_stream_tasks(watch_url).await
    }

    /// Queries the user's current account balance.
    ///
    /// # Returns
    ///
    /// On success, a [`Balance`] struct containing the user's balance information.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the API request fails.
    pub async fn get_balance(&self) -> Result<Balance, TripoError> {
        let url = self.base_url.join("user/balance")?;
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

    async fn connect_and_stream_tasks(
        &self,
        url: Url,
    ) -> Result<impl Stream<Item = Result<TaskStatus, TripoError>>, TripoError> {
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .method("GET")
            .uri(url.as_str())
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Host", url.host_str().unwrap_or_default())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .body(())?;

        let (ws_stream, _) = connect_async(request).await?;

        Ok(ws_stream.filter_map(|msg| async {
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<ApiResponse<TaskStatus>>(&text) {
                        Ok(api_response) => Some(Ok(api_response.data)),
                        Err(e) => Some(Err(TripoError::from(e))),
                    }
                }
                Ok(Message::Close(_)) => None,
                Err(e) => Some(Err(TripoError::from(e))),
                _ => None, // Ignore other message types like Binary, Ping, Pong
            }
        }))
    }

    fn get_ws_base_url(&self) -> Result<Url, TripoError> {
        let mut ws_url = self.base_url.clone();
        let scheme = if ws_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        ws_url
            .set_scheme(scheme)
            .map_err(|_| TripoError::ApiError {
                message: "Failed to set WebSocket scheme".to_string(),
            })?;
        Ok(ws_url)
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
    /// On success, the final [`TaskStatus`] of the completed or failed task.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if polling fails at any point.
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
    /// * `model_file` - A reference to a [`ResultFile`] struct containing the download URL.
    /// * `dest_dir` - The local directory path where the file will be saved.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PathBuf` of the newly created file.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if the download fails, the destination directory
    /// or file cannot be created, or there's an issue writing the file to disk.
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
    /// This is a convenience method that iterates over the results in a [`TaskStatus`]
    /// and downloads each available model file.
    ///
    /// # Arguments
    ///
    /// * `task_status` - The completed [`TaskStatus`] containing the models to download.
    /// * `dest_dir` - The directory where the models will be saved.
    ///
    /// # Returns
    ///
    /// A `Vec` containing the `PathBuf` of each downloaded file.
    ///
    /// # Errors
    ///
    /// Returns a `TripoError` if any of the model downloads fail.
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
