use crate::clients;
use crate::config::Options;
use crate::proxy;
use crate::proxy::client_response_handler;
use crate::proxy::{ParseError, RequestIdNotFound};
use crate::websocket_server::user_connected;
use crate::websocket_server::Users;
use serde::Serialize;
use std::error::Error;
use std::net::SocketAddr;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;
    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if err.find::<ParseError>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid ID Format";
    } else if err.find::<RequestIdNotFound>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "The supplied request_id was not found.";
    } else {
        eprintln!("unhandled rejection: {err:?}");
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }
    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });
    Ok(warp::reply::with_status(json, code))
}

pub async fn run_server(
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let config = Options::new();
    let log = warp::log::custom(|info| {
        println!(
            "Path: {} - Status: {} - Elapsed Time: {:?}",
            info.path(),
            info.status(),
            info.elapsed()
        );
    });
    let users = Users::default();
    let users = warp::any().map(move || users.clone());
    let proxy_route = warp::path!("api" / "proxy" / String)
        .and(warp::get())
        .and(users.clone())
        .and_then(proxy::handler);

    let response_route = warp::path!("api" / "proxy" / "response" / String)
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(client_response_handler);

    let clients_route = warp::path!("api" / "clients")
        .and(warp::get())
        .and(users.clone())
        .and_then(clients::handler);

    let ws_route = warp::path!("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(users.clone())
        .map(|ws: warp::ws::Ws, addr: Option<SocketAddr>, users| {
            ws.on_upgrade(move |socket| user_connected(socket, addr, users))
        });

    // Serve files from the "public" directory
    let static_route = warp::fs::dir("public").with(log);
    let routes = proxy_route
        .or(clients_route)
        .or(response_route)
        .or(ws_route)
        .or(static_route)
        .recover(handle_rejection);

    let host_vec: Vec<u8> = config
        .host
        .split('.')
        .map(|s| s.parse().expect("Failed to parse"))
        .collect();
    let ipv4_addr = Ipv4Addr::new(host_vec[0], host_vec[1], host_vec[2], host_vec[3]);
    let host = IpAddr::V4(ipv4_addr);

    // Bind the server with graceful shutdown, getting the address and the future
    let (addr, server_future) = warp::serve(routes).bind_with_graceful_shutdown(
        (host, config.http_server.port),
        shutdown_signal(running.clone()),
    );

    println!("Web server is listening on: http://{addr}");

    // Await the server future
    server_future.await;

    Ok(())
}

async fn shutdown_signal(running: Arc<AtomicBool>) {
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    println!("Shutting down static web server...");
}
