use tripo3d::{TripoClient, TaskState};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let client = TripoClient::new(None)?;

    // 1. Submit a task
    let prompt = "a high quality 3d model of a cat";
    println!("Submitting task for prompt: '{}'", prompt);
    let task_response = client.text_to_3d(prompt).await?;
    println!("Task submitted with ID: {}", task_response.task_id);

    // 2. Wait for the task to complete
    println!("\nWaiting for task to complete...");
    let final_status = client
        .wait_for_task(&task_response.task_id, true)
        .await?;

    // 3. Check final status and download models
    if final_status.status == TaskState::Success {
        println!("\nTask completed successfully!");
        
        let output_dir = "output";
        println!("Downloading models to '{}' directory...", output_dir);

        match client.download_all_models(&final_status, output_dir).await {
            Ok(downloaded_files) => {
                if downloaded_files.is_empty() {
                    println!("No models were generated or downloaded.");
                } else {
                    println!("\nSuccessfully downloaded {} files:", downloaded_files.len());
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
        println!("\nTask failed with status: {}", final_status.status);
    }

    Ok(())
} 