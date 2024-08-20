use colored::Colorize;
use crossterm::{execute, terminal::{Clear, ClearType}};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use url::Url;
use serde_json::Value;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::Instant;
use local_ip_address::local_ip;

fn clear_terminal() {
    execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear terminal");
}

async fn get_server_address(default_addr: String) -> String {
    loop {
        print!("Enter the server address (e.g., 127.0.0.1:1337) or press Enter to use default ({}): ", default_addr);
        io::stdout().flush().unwrap();

        let mut server_addr = String::new();
        io::stdin().read_line(&mut server_addr).expect("Failed to read input");

        let server_addr_trimmed = server_addr.trim();
        if server_addr_trimmed.is_empty() {
            return default_addr;
        } else {
            return server_addr_trimmed.to_string();
        }
    }
}

async fn connect_to_server(addr: String) -> (impl SinkExt<Message> + Unpin, impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin) {
    let url = Url::parse(&format!("ws://{}", addr)).expect("Invalid URL");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    ws_stream.split()
}

async fn receive_file_list(read: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin)) -> Vec<String> {
    if let Some(msg) = read.next().await {
        let msg = msg.expect("Failed to read message");

        if let Message::Text(text) = msg {
            let file_list: Value = serde_json::from_str(&text).expect("Failed to parse JSON");

            if let Some(files) = file_list.as_array() {
                println!("Received file list:");
                let file_names: Vec<String> = files
                    .iter()
                    .filter_map(|file| file.as_str().map(String::from))
                    .collect();

                for (index, file_name) in file_names.iter().enumerate() {
                    println!("{}: {}", index.to_string().green().bold(), file_name.magenta().bold());
                }
                return file_names;
            }
        }
    }
    vec![]
}

fn prompt_file_selection(files: &[String]) -> Option<String> {
    loop {
        print!("Enter the number of the file you want to download: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        match input.trim().parse::<usize>() {
            Ok(file_index) if file_index < files.len() => {
                clear_terminal();
                return Some(files[file_index].clone());
            }
            _ => println!("Invalid input, please enter a valid file number."),
        }
    }
}

async fn download_file(file_name: String, write: &mut (impl SinkExt<Message> + Unpin), read: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin)) {
    let _ = write.send(Message::Text(file_name.clone())).await;
    if let Some(file_msg) = read.next().await {
        let file_msg = file_msg.expect("Failed to read file message");
        if let Message::Binary(data) = file_msg {
            let total_size = data.len() as u64;
            let progress_bar: ProgressBar = ProgressBar::new(total_size);
            progress_bar.set_style(ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));
            progress_bar.set_message("Downloading");
            let start = Instant::now();
            let file_path = format!("Shared/{}", file_name);
            tokio::fs::write(&file_path, &data).await.expect("Failed to write file");
            let elapsed = start.elapsed().as_secs_f64();
            let speed = total_size as f64 / elapsed / 1024.0; // KB/s
            let message = format!("Downloaded at {:.2} KB/s", speed);
            progress_bar.finish_with_message(message.clone());
            println!("File received and saved as {}", file_path);
        }
    }
}

fn handle_user_choice() -> bool {
    loop {
        println!("{}", "1: Download another file".magenta().bold());
        println!("{}", "2: Connect to another server".magenta().bold());
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read input");
        match choice.trim() {
            "1" => return false, // Continue to download another file
            "2" => return true,  // Exit to connect to another server
            _ => println!("Invalid choice, please try again."),
        }
    }
}

async fn close_connection(write: &mut (impl SinkExt<Message> + Unpin)) {
    let _ = write.send(Message::Close(None)).await;
}

pub async fn run_client() {
    let local_ip = local_ip().expect("Failed to get local IP address");
    let default_addr = format!("{}:{}", local_ip, "1337");

    let mut server_address = get_server_address(default_addr.clone()).await;
    let (mut write_stream, mut read_stream) = connect_to_server(server_address.clone()).await;

    loop {
        println!("Connected to the server at {}", server_address);
        
        let files = receive_file_list(&mut read_stream).await;

        if files.is_empty() {
            println!("No files found. Press Enter to continue.");
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");
        } else {
            if let Some(file_name) = prompt_file_selection(&files) {
                download_file(file_name, &mut write_stream, &mut read_stream).await;
            }
        }

        if handle_user_choice() {
            close_connection(&mut write_stream).await; // Close the existing connection
            server_address = get_server_address(default_addr.clone()).await;
            let (new_write, new_read) = connect_to_server(server_address.clone()).await;
            write_stream = new_write;
            read_stream = new_read;
            println!("Connected to the server at {}", server_address);
        } else {
            let (new_write, new_read) = connect_to_server(server_address.clone()).await;
            write_stream = new_write;
            read_stream = new_read;
        }

        clear_terminal();
    }
}