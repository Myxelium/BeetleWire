use std::{fs, path::Path};

use colored::Colorize;
use local_ip_address::local_ip;
use tokio::task;

mod server;
mod client;

#[tokio::main]
async fn main() {
    // Create the Shared folder if it doesn't exist
    let shared_folder = "Shared";
    if !Path::new(shared_folder).exists() {
        fs::create_dir(shared_folder).expect("Failed to create Shared folder");
    }

    // Get the local IPv4 address
    let local_ip = local_ip().expect("Failed to get local IP address");
    let server = format!("{}:{}", local_ip, "1337");

    // Use server address here
    println!("{}", format!("Hosting files at address: {}", server).green());

    let file_path: &str = shared_folder;
    let server: task::JoinHandle<()> = task::spawn(server::run_server(server.to_string().clone(), file_path));
    let client: task::JoinHandle<()> = task::spawn(client::run_client());

    let _ = tokio::join!(server, client);
}