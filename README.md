# Tripo3D Rust SDK

[![crates.io](https://img.shields.io/crates/v/tripo3d.svg)](https://crates.io/crates/tripo3d)
[![docs.rs](https://docs.rs/tripo3d/badge.svg)](https://docs.rs/tripo3d)

An unofficial Rust SDK for the [Tripo3d API](https://platform.tripo3d.ai/documentation/guides/get_started), providing an easy-to-use interface for 3D model generation.

## Features

- **Text-to-3D**: Generate 3D models from text prompts.
- **Image-to-3D**: Generate 3D models from images.
- **Asynchronous API**: Fully async support for efficient, non-blocking operations.
- **Account Balance**: Check your account balance.
- **Easy to Use**: High-level abstractions to simplify your code.

## Getting Started

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tripo3d = "0.1.0"
```

### Authentication

The SDK requires an API key for authentication. You can pass the key directly to the client or set the `TRIPO_API_KEY` environment variable. You can find your API key on the [Tripo3D Platform](https://platform.tripo3d.ai/account/api_keys).

### Usage

Here's an example of the complete workflow: submitting a task, waiting for it to complete, and downloading the resulting model.

```rust
use tripo3d::{TripoClient, TaskState};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from an environment variable.
    let client = TripoClient::new(env::var("TRIPO_API_KEY").ok())?;

    // 1. Submit a task.
    let prompt = "a high quality 3d model of a cat";
    println!("Submitting task for prompt: '{}'", prompt);
    let task_response = client.text_to_3d(prompt).await?;
    println!("Task submitted with ID: {}", task_response.task_id);

    // 2. Wait for the task to complete, with verbose progress.
    println!("\nWaiting for task to complete...");
    let final_status = client
        .wait_for_task(&task_response.task_id, true)
        .await?;

    // 3. Check the final status and download the models if successful.
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
```

For more detailed examples, please see the `examples` directory in the repository.

## Documentation

For more detailed information about the API and its features, please refer to the [official documentation](https://platform.tripo3d.ai/documentation/guides/get_started).

## License

This SDK is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details. 