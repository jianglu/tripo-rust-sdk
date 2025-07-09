//! This example demonstrates how to query the status of a specific task.
//!
//! It initializes a `TripoClient`, takes a task ID from the command-line arguments,
//! and calls the `get_task` method to retrieve the task's current status.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage:
//! `cargo run --example get_task <TASK_ID>`

use std::env;
use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    // Get the task ID from command-line arguments.
    let task_id = env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Please provide a task ID as a command-line argument."))?;

    println!("Querying status for task: {}", task_id);

    // Call the get_task method.
    match client.get_task(&task_id).await {
        Ok(status) => {
            println!("\nSuccessfully retrieved task status:");
            println!("{:#?}", status);
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }

    Ok(())
} 