use anyhow::Result;
use futures_util::StreamExt;
use tripo3d::TripoClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file if it exists.
    dotenvy::dotenv().ok();

    // Make sure to set the TRIPO_API_KEY environment variable.
    println!("Watching all tasks... Press Ctrl+C to stop.");

    let client = TripoClient::new(None)?;

    // Watch all tasks starting from now.
    // To watch tasks from a specific time, you can pass a DateTime<Utc> object.
    // For example, to watch tasks from the last 24 hours:
    // let since = chrono::Utc::now() - chrono::Duration::days(1);
    // let mut stream = client.watch_all_tasks(Some(since)).await?;
    let stream = client.watch_all_tasks(None).await?;
    let mut pinned_stream = Box::pin(stream);

    while let Some(task_status_result) = pinned_stream.next().await {
        match task_status_result {
            Ok(task_status) => {
                println!("--- Task Update ---");
                println!("  Task ID: {}", task_status.task_id);
                println!("  Status: {:?}", task_status.status);
                println!("  Progress: {}%", task_status.progress);
                if let Some(model) = &task_status.result.glb_model {
                    println!("  Model URL: {}", model.url);
                }
                println!("-------------------");
            }
            Err(e) => {
                eprintln!("An error occurred while watching tasks: {}", e);
                break;
            }
        }
    }

    Ok(())
} 