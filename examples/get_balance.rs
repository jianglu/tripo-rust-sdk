use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let client = TripoClient::new(None)?;

    println!("Querying account balance...");

    match client.get_balance().await {
        Ok(balance) => {
            println!("Account balance details:");
            println!("  Available: {}", balance.balance);
            println!("  Frozen: {}", balance.frozen);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
} 