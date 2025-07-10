//! This example demonstrates the standard file upload method.
//!
//! It initializes a `TripoClient` and calls the `upload_file` method.
//! This method sends the file directly to the Tripo API and returns a `file_token`
//! which can be used in other API calls that require a file token.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set,
//! and an image file must exist at `assets/image.png`.
//!
//! Usage:
//! `cargo run --example upload_image`

use tripo3d::TripoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(env::var("TRIPO_API_KEY").ok())?;

    // Define the path to the input image.
    let image_path = "assets/image.png";
    println!("Uploading local image via standard method: '{}'", image_path);

    // Call the upload_file method.
    match client.upload_file(image_path).await {
        Ok(file_token) => {
            println!("\nSuccessfully uploaded file.");
            println!("-> File Token: {}", file_token);
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }

    Ok(())
} 