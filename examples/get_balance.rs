//! This example demonstrates how to query the account balance.
//!
//! It initializes a `TripoClient` and calls the `get_balance` method to retrieve
//! the current balance and frozen credit information.
//!
//! To run this example, you must have the `TRIPO_API_KEY` environment variable set.
//!
//! Usage:
//! `cargo run --example get_balance`

use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Initialize the client from the TRIPO_API_KEY environment variable.
    let client = TripoClient::new(None)?;

    println!("Querying account balance...");

    // Call the get_balance method.
    match client.get_balance().await {
        Ok(balance) => {
            println!("\nSuccessfully retrieved balance:");
            println!("{:#?}", balance);
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }

    Ok(())
} 