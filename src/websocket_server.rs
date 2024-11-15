use futures_util::{SinkExt, StreamExt, TryFutureExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub struct Client {
    pub addr: Option<SocketAddr>,
    pub sender: mpsc::UnboundedSender<Message>,
}

pub type Users = Arc<RwLock<HashMap<usize, Client>>>;

pub async fn user_connected(ws: WebSocket, addr: Option<SocketAddr>, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    if let Some(addr) = addr {
        println!("Client connected from {}:{}", addr.ip(), addr.port());
    }

    eprintln!("new chat user: {my_id}");

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {e}");
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users
        .write()
        .await
        .insert(my_id, Client { addr, sender: tx });

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={my_id}): {e}");
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

pub async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let Ok(msg) = msg.to_str() else {
        return;
    };

    let new_msg = format!("<User#{my_id}>: {msg}");

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, client) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = client.sender.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

pub async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {my_id}");

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
