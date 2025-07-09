//! This example demonstrates a wait-and-download workflow:
//! 1. Taking an existing task ID from the command line.
//! 2. Polling the task status until it completes.
//! 3. Downloading the resulting model to a temporary directory if the task was successful.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage:
//! `cargo run --example wait_and_download <TASK_ID>`

use tripo3d::{TaskState, TripoClient};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    // 1. Get the task ID from command-line arguments.
    let task_id = env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Please provide a task ID as a command-line argument."))?;

    // 2. Wait for the task to complete
    println!("\nWaiting for task `{}` to complete...", task_id);
    let final_status = match client
        .wait_for_task(&task_id, true)
        .await
    {
        Ok(status) => status,
        Err(e) => {
            eprintln!("\nError waiting for task: {}", e);
            return Ok(()); // Exit gracefully
        }
    };

    // 3. Check final status and download models to a temporary directory
    if final_status.status == TaskState::Success {
        let temp_dir = tempfile::Builder::new()
            .prefix("tripo_download_")
            .tempdir()?;
        println!(
            "\nTask completed successfully! Downloading model(s) to temporary directory: {}",
            temp_dir.path().display()
        );

        match client
            .download_all_models(&final_status, temp_dir.path())
            .await
        {
            Ok(downloaded_files) => {
                if downloaded_files.is_empty() {
                    println!("\nNo models were available for download.");
                } else {
                    println!("\nSuccessfully downloaded {} file(s):", downloaded_files.len());
                    for path in downloaded_files {
                        println!("- {}", path.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("\nFailed to download models: {}", e);
            }
        }
    } else {
        println!(
            "\nTask finished with non-success status: {}",
            final_status.status
        );
    }

    Ok(())
} 