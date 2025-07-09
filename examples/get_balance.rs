use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let client = TripoClient::new(None)?;

    println!("Querying account balance...");

    match client.get_balance().await {
        Ok(balance) => {
            println!("Account Balance: {:?}", balance);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
} 