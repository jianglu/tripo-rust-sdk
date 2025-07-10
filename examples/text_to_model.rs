//! This example demonstrates how to submit a text-to-3D task.
//!
//! It initializes a `TripoClient` and calls the `text_to_3d` method with a sample prompt.
//! The resulting task ID is then printed to the console.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage: `cargo run --example text_to_model -- "a high quality armchair"`

use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // 1. Initialize the client.
    // The client will automatically read the `TRIPO_API_KEY` environment variable.
    let client = TripoClient::new(None)?;

    // 2. Get the prompt from command-line arguments or use a default.
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "a high quality armchair".to_string());

    // 3. Call the text_to_model API.
    println!("Submitting task for prompt: \"{}\"...", prompt);
    match client.text_to_model(&prompt).await {
        Ok(task) => {
            println!("Task submitted successfully!");
            println!("-> Task ID: {}", task.task_id);
        }
        Err(e) => {
            eprintln!("API call failed: {}", e);
        }
    }

    Ok(())
} 