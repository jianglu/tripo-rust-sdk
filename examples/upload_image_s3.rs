//! This example demonstrates how to directly upload a local image file via S3.
//!
//! It initializes a `TripoClient` and calls the `upload_file_s3` method with a local image path.
//! This method handles the temporary S3 upload and returns a `FileContent` struct,
//! which can be used in other API calls that require a file.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set,
//! and an image file must exist at `assets/image.png`.
//!
//! Usage:
//! `cargo run --example upload_image_sts`

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
    println!("Uploading local image via S3: '{}'", image_path);

    // Call the upload_file_s3 method.
    match client.upload_file_s3(image_path).await {
        Ok(file_content) => {
            println!("\nSuccessfully uploaded file.");
            println!("-> File Content: {:#?}", file_content);
            // This file_content struct can now be used to create a task,
            // for example by creating a token from its `object.key` and passing it to `image_to_model`,
            // but this example focuses only on the upload part.
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }

    Ok(())
} 