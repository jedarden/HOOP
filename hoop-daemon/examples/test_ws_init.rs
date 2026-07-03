//! Quick test to see what the actual WebSocket init message looks like
use hoop_daemon::tests::integration_harness::spawn_test_daemon;
use futures_util::stream::StreamExt;
use tokio_tungstenite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test daemon...");
    let (base_url, _shutdown, _temp_dir) = spawn_test_daemon().await?;

    let ws_url = base_url.replace("http://", "ws://");
    let ws_url = format!("{}/ws", ws_url);

    println!("Connecting to WebSocket at: {}", ws_url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;

    let (_, mut ws_receiver) = ws_stream.split();

    // Read first 5 messages
    for i in 0..5 {
        match ws_receiver.next().await {
            Some(Ok(msg)) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    let json: serde_json::Value = serde_json::from_str(&text)?;
                    println!("Message {}: type = {}", i, json.get("type").unwrap_or(&serde_json::Value::Null));

                    if i == 0 {
                        println!("  Full init event: {}", serde_json::to_string_pretty(&json)?);
                        if let Some(subs) = json.get("subscriptions") {
                            println!("  subscriptions: {}", subs);
                        }
                    }
                }
            }
            other => println!("Message {}: {:?}", i, other),
        }
    }

    println!("Test completed successfully");
    Ok(())
}
