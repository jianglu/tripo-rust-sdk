//! This example demonstrates how to submit an image-to-3D task.
//!
//! It initializes a `TripoClient` and calls the `image_to_3d` method with a sample image.
//! The resulting task ID is then printed to the console.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set,
//! and an image file must exist at `assets/hamburger.png`.
//!
//! Usage:
//! `cargo run --example image_to_3d`

use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    // Define the path to the input image.
    let image_path = "assets/image.png";
    println!("Submitting task for image: '{}'", image_path);

    // Call the image_to_3d method.
    match client.image_to_3d(image_path).await {
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