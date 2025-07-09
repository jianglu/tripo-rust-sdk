use tripo3d::TripoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize the client from environment variable
    let client = TripoClient::new(None)?;

    // Example: Image-to-3D
    let image_path = "assets/image.png";
    println!("Submitting task for image: '{}'", image_path);

    match client.image_to_3d(image_path).await {
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