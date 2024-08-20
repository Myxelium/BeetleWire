use futures_util::{StreamExt, TryStreamExt}; // Add TryStreamExt
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;

pub async fn run_client(addr: &str) {
    let url = Url::parse(&format!("ws://{}", addr)).expect("Invalid URL");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (_, mut read) = ws_stream.split(); // Use try_split instead of split

    while let Some(msg) = read.next().await {
        let msg = msg.expect("Failed to read message");
        if let Message::Binary(data) = msg {
            tokio::fs::write("received_file.png", data).await.expect("Failed to write file");
            println!("File received and saved as received_file.png");
        }
    }
}