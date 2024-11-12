use std::env;

pub struct ConnectOptions {
    pub port: u16,
}

pub struct HubProps {
    pub proxy_response_uri: String,
    pub ws_uri: String,
}

pub struct Options {
    pub host: String,
    pub hub: HubProps,
    pub http_server: ConnectOptions,
}

impl Options {
    pub fn new() -> Self {
        let http_server_port = env::var("HTTP_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8890);

        let host = env::var("WS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let ws_uri =
            env::var("HUB_WS_URI").unwrap_or_else(|_| "ws://127.0.0.1:8890/ws".to_string());

        let proxy_response_uri = env::var("HUB_PROXY_RESPONSE_URI")
            .unwrap_or_else(|_| "http://127.0.0.1:8890/api/proxy/response".to_string());

        let http_server = ConnectOptions {
            port: http_server_port,
        };

        Options {
            host,
            hub: HubProps {
                proxy_response_uri,
                ws_uri,
            },
            http_server,
        }
    }
}
