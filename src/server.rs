use std::path::Path;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{fs::{self, File}, io::AsyncReadExt, net::TcpListener};
use tokio_tungstenite::accept_async;
use tungstenite::Message;

pub async fn run_server(addr: String, dir_path: &str) {
    let listener = TcpListener::bind(addr.clone()).await.expect("Failed to bind");
    println!("Server listening on {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        let dir_path = dir_path.to_string();
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");
            let (mut write, mut read) = ws_stream.split();
            // Debugging output to verify the directory path
            println!("Reading directory: {}", dir_path);
            // List files in the directory
            let mut file_list = Vec::new();
            match fs::read_dir(&dir_path).await {
                Ok(mut entries) => {
                    while let Some(entry) = entries.next_entry().await.expect("Failed to read entry") {
                        if entry.path().is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                file_list.push(file_name.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read directory: {:?}", e);
                    return;
                }
            }
            // Send the list of files to the client
            let file_list_json = json!(file_list).to_string();
            write.send(Message::Text(file_list_json)).await.expect("Failed to send file list");
            // Wait for the client to request a file
            if let Some(msg) = read.next().await {
                let msg = msg.expect("Failed to read message");
                if let Message::Text(file_name) = msg {
                    let file_path = Path::new(&dir_path).join(file_name);
                    let mut file = File::open(file_path).await.expect("Failed to open file");
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).await.expect("Failed to read file");
                    // Send the requested file to the client
                    write.send(Message::Binary(buffer)).await.expect("Failed to send file");
                }
            }
        });
    }
}