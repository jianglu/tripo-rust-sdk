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

pub mod client;
pub mod error;
pub mod types;

pub use client::TripoClient;
pub use error::TripoError;
pub use types::{Balance, ResultFile, TaskResponse, TaskResult, TaskState, TaskStatus}; 