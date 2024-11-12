mod cleanup;
mod cli;
mod clients;
mod config;
mod cpu_monitor;
mod db;
mod proxy;
mod utils;
mod warp_server;
mod websocket_client;
mod websocket_server;

use clap::Parser;
use cli::{Args, Commands};
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dotenv::dotenv().ok();
    let running = Arc::new(AtomicBool::new(true));
    let running_ctrlc = running.clone();
    let ctrlc_task = tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        println!("Received Ctrl+C, initiating graceful shutdown...");
        running_ctrlc.store(false, Ordering::SeqCst);
    });

    match &args.command {
        Some(Commands::Client {}) => {
            println!("Running the Client program");
            db::init()?;
            let cpu_task = tokio::spawn(cpu_monitor::cpu_monitoring_loop(running.clone()));
            let cleanup_task = tokio::spawn(cleanup::run(running.clone()));
            let websocket_task =
                tokio::spawn(websocket_client::connect_with_retry(running.clone()));
            let _ = tokio::join!(cpu_task, cleanup_task, websocket_task, ctrlc_task);
        }
        Some(Commands::Hub {}) => {
            println!("Running the Hub program");
            let webserver_task = tokio::spawn(warp_server::run_server(running.clone()));
            let _ = tokio::join!(webserver_task);
        }
        None => {
            println!("Invalid subcommand. See usage.");
        }
    }

    Ok(())
}
