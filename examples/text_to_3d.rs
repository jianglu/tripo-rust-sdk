use tripo3d::TripoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize the client from environment variable
    let client = TripoClient::new(None)?;

    // Example: Text-to-3D
    let prompt = "a delicious hamburger";
    println!("Submitting task for prompt: '{}'", prompt);

    match client.text_to_3d(prompt).await {
        Ok(response) => {
            println!("Successfully submitted task.");
            println!("Task ID: {}", response.task_id);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
} 