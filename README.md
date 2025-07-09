# Tripo3D Rust SDK

[![crates.io](https://img.shields.io/crates/v/tripo3d.svg)](https://crates.io/crates/tripo3d)
[![docs.rs](https://docs.rs/tripo3d/badge.svg)](https://docs.rs/tripo3d)

The official Rust SDK for the [Tripo3d API](https://platform.tripo3d.ai/documentation/guides/get_started), providing an easy-to-use interface for 3D model generation.

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

Here's a quick example of how to generate a 3D model from a text prompt:

```rust
use tripo3d::TripoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize the client
    let api_key = env::var("TRIPO_API_KEY").expect("TRIPO_API_KEY must be set");
    let client = TripoClient::new(Some(api_key));

    // Example: Text-to-3D
    let prompt = "a delicious hamburger";
    let response = client.text_to_3d(prompt).await?;
    println!("Task ID: {}", response.task_id);

    // You can then use the task_id to query for the result
    // ...

    Ok(())
}
```

For more detailed examples, please see the `examples` directory in the repository.

## Documentation

For more detailed information about the API and its features, please refer to the [official documentation](https://platform.tripo3d.ai/documentation/guides/get_started).

## License

This SDK is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details. 