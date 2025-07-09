use thiserror::Error;

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