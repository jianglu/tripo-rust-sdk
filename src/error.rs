#[derive(Debug, thiserror::Error)]
pub enum TripoError {
    #[error("API key is missing. Please provide it or set the TRIPO_API_KEY environment variable.")]
    MissingApiKey,
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse API response: {0}")]
    ResponseParseFailed(#[from] serde_json::Error),
    #[error("API request failed: {message}")]
    ApiError { message: String },
    #[error("URL parsing failed: {0}")]
    UrlParseFailed(#[from] url::ParseError),
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("File upload failed: {0}")]
    UploadError(#[from] aws_sdk_s3::primitives::ByteStreamError),
} 