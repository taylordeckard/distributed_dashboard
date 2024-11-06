use crate::config::Options;
use crate::utils;
use crate::db::get_all_stats;
use futures_util::{SinkExt, StreamExt};
use std::{
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite, tungstenite::protocol::Message};
use serde::Deserialize;
use serde_json::from_str;
use reqwest::Client;


#[derive(Debug, Deserialize)]
struct ReqMsg {
    request_id: String,
}

async fn sleep_until_interrupted(
    delay: Duration,
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut elapsed = Duration::from_secs(0);
    let interval = Duration::from_secs(1);

    while elapsed < delay {
        if !running.load(Ordering::SeqCst) {
            return Err(Box::<dyn Error + Send + Sync>::from(
                "Interrupted during sleep",
            ));
        }
        sleep(interval).await;
        elapsed += interval;
    }

    Ok(())
}

async fn handle_messages(
    mut read: impl StreamExt<Item = Result<Message, tungstenite::Error>> + Unpin,
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(msg)) => match msg {
                        Message::Text(text) => {
                            println!("Received: {text}");
                            let result = from_str::<ReqMsg>(&text).unwrap();
                            let client = Client::new();
                            match get_all_stats() {
                                Ok(response) => {
                                    let config = Options::new();
                                    let server_url = format!(
                                        "http://{}:{}/response/{}",
                                        config.host,
                                        config.http_server.port,
                                        result.request_id,
                                    );
                                    let res = client
                                        .post(server_url)
                                        .json(&response)
                                        .send()
                                        .await?;
                                    println!("{}", res.text().await?);
                                },
                                Err(_e) => {
                                    eprintln!("An error occurred");
                                }
                            }
                        },
                        Message::Binary(data) => println!("Received binary data: {data:?}"),
                        Message::Ping(_) => println!("Received ping"),
                        Message::Pong(_) => println!("Received pong"),
                        Message::Close(_) => {
                            println!("Server closed connection");
                            return Ok(());
                        },
                        Message::Frame(_) => println!("Received raw frame"),
                    },
                    Some(Err(e)) => {
                        eprintln!("Error receiving message: {e}");
                        return Err(Box::<dyn Error + Send + Sync>::from("Error occurred receiving message"));
                    },
                    None => {
                        println!("WebSocket stream closed");
                        break;
                    }
                }
            }
            () = utils::wait_for_running_to_be_false(running.clone()) => {
                println!("Receive task interrupted");
                break;
            }
        }
    }
    Ok(())
}

async fn send_test_message(
    write: &mut (impl SinkExt<Message> + Unpin),
) -> Result<(), Box<dyn Error + Send + Sync>> {
    write
        .send(Message::Text("Hello, WebSocket Server!".to_string()))
        .await
        .map_err(|_| {
            println!("Error sending message");
            Box::<dyn Error + Send + Sync>::from("Error sending test message")
        })
}

pub async fn connect_with_retry(
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut retry_count = 0;
    let max_retries = None; // Set to None for infinite retries
    let config = Options::new();

    loop {
        if !running.load(Ordering::SeqCst) {
            println!("Shutting down WebSocket client...");
            break;
        }

        let url = format!("ws://{}/ws", config.ws_server_address());
        println!("Attempting to connect to {url}");

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                println!("WebSocket connection established");
                let (mut write, read) = ws_stream.split();

                // Reset retry count on successful connection
                retry_count = 0;

                let receive_task = tokio::spawn(handle_messages(read, running.clone()));

                // Send a test message with error handling
                if let Err(e) = send_test_message(&mut write).await {
                    eprintln!("Error sending message: {e}");
                }

                // Wait for the receive task to complete or error
                match receive_task.await {
                    Ok(Ok(())) => println!("Connection closed gracefully"),
                    Ok(Err(e)) => eprintln!("Connection error: {e}"),
                    Err(e) => eprintln!("Task error: {e}"),
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {e}");
            }
        }

        // Handle reconnection
        retry_count += 1;
        if let Some(max) = max_retries {
            if retry_count >= max {
                eprintln!("Max retry attempts ({max}) reached. Exiting.");
                return Err(Box::<dyn Error + Send + Sync>::from("Max retries exceeded"));
            }
        }

        if !running.load(Ordering::SeqCst) {
            println!("Shutting down WebSocket client...");
            break;
        }

        // Exponential backoff: 1s, 2s, 4s, 8s, etc.
        let delay = Duration::from_secs(2u64.pow(retry_count - 1));
        println!("Retrying in {} seconds...", delay.as_secs());
        sleep_until_interrupted(delay, running.clone()).await?;
    }

    Ok(())
}
