use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::env;
use thiserror::Error;
use url::Url;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use reqwest::multipart;

const DEFAULT_API_URL: &str = "https://api.tripo3d.ai/";

#[derive(Error, Debug)]
pub enum TripoError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("API key not provided")]
    NoApiKey,
    #[error("API error: {message}")]
    ApiError { message: String },
}

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

#[derive(Deserialize, Debug)]
pub struct TaskResponse {
    #[serde(rename = "task_id")]
    pub task_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Model {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct TaskStatus {
    pub task_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub progress: u32,
    pub created_at: String,
    pub models: Option<Vec<Model>>,
}

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub total_granted_credits: f64,
    pub total_used_credits: f64,
    pub total_available_credits: f64,
}


impl TripoClient {
    pub fn new(api_key: Option<String>) -> Result<Self, TripoError> {
        let api_key = api_key.or_else(|| env::var("TRIPO_API_KEY").ok()).ok_or(TripoError::NoApiKey)?;
        Self::new_with_url(api_key, DEFAULT_API_URL)
    }

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

    pub async fn text_to_3d(&self, prompt: &str) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/direct/generate")?;
        let request_body = TextTo3DRequest { prompt, type_: "text_to_model" };

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

    pub async fn image_to_3d<P: AsRef<Path>>(&self, image_path: P) -> Result<TaskResponse, TripoError> {
        let url = self.base_url.join("v2/direct/generate")?;

        let file = File::open(image_path).await.map_err(|e| TripoError::ApiError{ message: e.to_string() })?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let some_file = multipart::Part::stream(file_body)
            .file_name("image.png")
            .mime_str("image/png")?;

        let form = multipart::Form::new()
            .text("type", "image_to_model")
            .part("file", some_file);

        let response = self.client.post(url)
            .multipart(form)
            .send()
            .await?;

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

    pub async fn get_task(&self, task_id: &str) -> Result<TaskStatus, TripoError> {
        let url = self.base_url.join(&format!("v2/organization/tasks/{}", task_id))?;

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
} 