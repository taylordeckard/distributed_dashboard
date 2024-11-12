use crate::websocket_server::Users;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Client {
    address: String,
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct Response {
    clients: Vec<Client>,
}

pub async fn handler(users: Users) -> Result<impl warp::Reply, warp::Rejection> {
    let user_map = users.read().await;
    let client_ids: Vec<usize> = user_map.keys().copied().collect();
    let clients = client_ids
        .into_iter()
        .map(|id| Client {
            address: format!(
                "{}:{}",
                user_map.get(&id).unwrap().addr.unwrap().ip(),
                user_map.get(&id).unwrap().addr.unwrap().port(),
            ),
            id,
        })
        .collect();
    let res = Response { clients };

    Ok(warp::reply::json(&res))
}
