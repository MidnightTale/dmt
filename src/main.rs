use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::connect_async;
use tokio::time::{sleep, Duration};

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY: u64 = 1000;

#[tokio::main]
async fn main() {
    let uri = "ws://localhost:6123";
    let mut retry_count = 0;

    while retry_count < MAX_RETRIES {
        match connect_and_process(uri).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("Connection error: {}, retrying...", e);
                sleep(Duration::from_millis(RETRY_DELAY)).await;
                retry_count += 1;
            }
        }
    }
}

async fn connect_and_process(uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(uri).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to window management events
    write.send("sub -e window_managed".into()).await?;

    // Process incoming messages
    while let Some(msg) = read.next().await {
        let msg = msg?;
        let json_response: Value = serde_json::from_str(&msg.to_string())?;
        
        // Extract data using chained get() calls with early return on None
        let size = json_response
            .get("data")
            .and_then(|data| data.get("managedWindow"))
            .and_then(|window| window.get("tilingSize"))
            .and_then(|size| size.as_f64());

        if let Some(size) = size {
            if size <= 0.5 {
                write.send("command toggle-tiling-direction".into()).await?
            }
        }
    }

    Ok(())
}
