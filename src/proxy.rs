use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use warp::ws::Message;
use crate::websocket_server::Users;
use std::collections::HashMap;
use uuid::Uuid;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};


#[derive(Serialize, Deserialize)]
struct ProxyResponse {
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct ClientMessage {
    request_id: String,
}

#[derive(Debug)]
pub struct ParseError;

#[derive(Debug)]
pub struct RequestIdNotFound;

impl warp::reject::Reject for ParseError {}
impl warp::reject::Reject for RequestIdNotFound {}

static PENDING_REQUESTS: Lazy<RwLock<HashMap<String, Option<String>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

static MAX_POLL_ITERS: u32 = 5000;

async fn poll_for_response(uuid: String) -> Option<String> {
    for _ in 1..MAX_POLL_ITERS {
        let req_map = PENDING_REQUESTS.read().await;
        if let Some(response) = req_map.get(&uuid).cloned() {
            if response.is_some() {
                return response;
            }
        }
        drop(req_map);
        sleep(Duration::from_millis(100)).await; 
    }
    None
}

pub async fn handler(source: String, users: Users) -> Result<impl warp::Reply, warp::Rejection> {
    let id_str = source.clone();
    let id: usize = id_str.parse().map_err(|_e| warp::reject::custom(ParseError))?;
    let user_map = users.read().await;
    let user = user_map.get(&id);
    if let Some(user) = user {
        let body = ClientMessage {
            request_id: Uuid::new_v4().to_string(),
        };
        let mut req_map = PENDING_REQUESTS.write().await;
        req_map.insert(body.request_id.clone(), None);
        drop(req_map);
        let msg = Message::text(serde_json::to_string(&body).unwrap());
        if let Err(_disconnected) = user.sender.send(msg) {
            eprintln!("Could not reach client through websocket.");
        };
        let response = poll_for_response(body.request_id.clone()).await;
        let json_res: Value = serde_json::from_str(&response.unwrap())
            .expect("Expected response to be valid JSON");
        Ok(warp::reply::json(&json_res))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn client_response_handler(
    request_id: String,
    body: warp::hyper::body::Bytes
) -> Result<impl warp::Reply, warp::Rejection> {
    let data = std::str::from_utf8(&body).unwrap();
    let mut req_map = PENDING_REQUESTS.write().await;
    if req_map.get(&request_id).is_none() {
        eprintln!("here");
        drop(req_map);
        return Err(warp::reject::custom(RequestIdNotFound));
    }
    req_map.insert(request_id, Some(data.to_string()));
    Ok(warp::reply::html("Success"))
}
