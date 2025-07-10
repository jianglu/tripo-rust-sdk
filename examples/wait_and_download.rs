//! This example demonstrates a wait-and-download workflow:
//! 1. Taking an existing task ID from the command line.
//! 2. Polling the task status until it completes.
//! 3. Downloading the resulting model to a specified or temporary directory.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage:
//! `cargo run --example wait_and_download <TASK_ID> [OUTPUT_DIR]`
//!
//! Arguments:
//! - `<TASK_ID>`: The ID of the task to monitor.
//! - `[OUTPUT_DIR]`: Optional. The directory to save the downloaded models. Defaults to a temporary directory.

use tripo3d::{TaskState, TripoClient, TaskStatus};
use std::env;
use std::path::{Path, PathBuf};

async fn download_and_report(client: &TripoClient, task_status: &TaskStatus, output_dir: &Path) -> anyhow::Result<()> {
    println!(
        "\nTask completed successfully! Downloading model(s) to '{}'...",
        output_dir.display()
    );

    match client.download_all_models(task_status, output_dir).await {
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
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    // 1. Get the task ID from command-line arguments.
    let task_id = env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: cargo run --example wait_and_download <TASK_ID> [OUTPUT_DIR]"))?;
    
    // Wait for the task to complete
    println!("\nWaiting for task `{}` to complete...", task_id);
    let final_status = match client.wait_for_task(&task_id, true).await {
        Ok(status) => status,
        Err(e) => {
            eprintln!("\nError waiting for task: {}", e);
            return Ok(()); // Exit gracefully
        }
    };

    // 4. Check final status and download models
    if final_status.status != TaskState::Success {
        println!("\nTask finished with status: {:?}", final_status.status);
        return Ok(());
    }

    if let Some(output_dir_str) = env::args().nth(2) {
        // Case 1: An output directory was provided.
        download_and_report(&client, &final_status, &PathBuf::from(output_dir_str)).await?;
    } else {
        // Case 2: No output directory, use a temporary one.
        let temp_dir = tempfile::Builder::new()
            .prefix("tripo_download_")
            .tempdir()?;
        download_and_report(&client, &final_status, temp_dir.path()).await?;
    }

    Ok(())
} 