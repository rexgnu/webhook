mod config;
mod request;
mod server;
mod ui;

use config::Config;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load();
    let listening_address = config.address();

    // Create channel for request communication
    let (tx, rx) = mpsc::unbounded_channel();

    // Spawn HTTP server in background
    let server_config = config.clone();
    tokio::spawn(async move {
        if let Err(e) = server::run_server(server_config, tx).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Run TUI on main thread
    ui::run_tui(listening_address, rx).await?;

    Ok(())
}
