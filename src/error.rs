use thiserror::Error;

/// The primary error type for the Tripo3D SDK.
#[derive(Debug, Error)]
pub enum TripoError {
    /// The API key was not provided.
    /// It must be supplied during client creation or set via the `TRIPO_API_KEY` environment variable.
    #[error("API key is missing. Please provide it or set the TRIPO_API_KEY environment variable.")]
    MissingApiKey,

    /// A network request failed. This is often a wrapper around a `reqwest::Error`.
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Failed to parse a JSON response from the API.
    #[error("Failed to parse API response: {0}")]
    ResponseParseError(#[from] serde_json::Error),

    /// The Tripo3D API returned an error. The message contains the details from the API.
    #[error("API request failed: {message}")]
    ApiError { message: String },

    /// A URL could not be parsed. This can happen with an invalid base URL or a malformed URL from the API.
    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),

    /// An I/O error occurred, typically when reading or writing files.
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The byte stream for a file upload could not be created.
    #[error("File upload stream could not be created: {0}")]
    UploadStreamError(#[from] aws_sdk_s3::primitives::ByteStreamError),

    /// A WebSocket connection or message error occurred.
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// An HTTP request could not be built.
    #[error("Failed to build HTTP request: {0}")]
    HttpError(#[from] tokio_tungstenite::tungstenite::http::Error),
} 