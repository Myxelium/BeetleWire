use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn run_server(addr: &str, file_path: &str) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let file_path = file_path.to_string();
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");
            let (mut write, _) = ws_stream.split();

            let path = Path::new(&file_path);
            let mut file = File::open(path).await.expect("Failed to open file");
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await.expect("Failed to read file");

            write.send(Message::Binary(buffer)).await.expect("Failed to send file");
        });
    }
}