mod server;
mod client;

use tokio::task;

#[tokio::main]
async fn main() {
    let server_addr = "127.0.0.1:8080";
    let file_path = "c:/shared/file.jpg";

    let server = task::spawn(server::run_server(server_addr, file_path));
    let client = task::spawn(client::run_client(server_addr));

    let _ = tokio::join!(server, client);
}