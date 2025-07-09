use tripo3d::TripoClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let client = TripoClient::new(None)?;

    let task_id = env::args().nth(1).expect("Please provide a task ID");

    println!("Querying status for task: {}", task_id);

    match client.get_task(&task_id).await {
        Ok(status) => {
            println!("Task Status: {:?}", status);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
} 