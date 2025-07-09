//! This example demonstrates how to submit a text-to-3D task.
//!
//! It initializes a `TripoClient` and calls the `text_to_3d` method with a sample prompt.
//! The resulting task ID is then printed to the console.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage:
//! `cargo run --example text_to_3d`

use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    // Define the prompt for the 3D model.
    let prompt = "a nice house";
    println!("Submitting task for prompt: '{}'", prompt);

    // Call the text_to_3d method.
    match client.text_to_3d(prompt).await {
        Ok(response) => {
            println!("\nSuccessfully submitted task.");
            println!("-> Task ID: {}", response.task_id);
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }

    Ok(())
} 