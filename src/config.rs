use std::env;

pub struct ConnectOptions {
    pub port: u16,
}

pub struct Options {
    pub host: String,
    pub ws_server: ConnectOptions,
    pub http_server: ConnectOptions,
}

impl Options {
    pub fn new() -> Self {
        let ws_server_port = env::var("WS_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8890);

        let http_server_port = env::var("HTTP_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8890);

        let host = env::var("WS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let ws_server = ConnectOptions {
            port: ws_server_port,
        };
        let http_server = ConnectOptions {
            port: http_server_port,
        };

        Options {
            host,
            ws_server,
            http_server,
        }
    }

    pub fn ws_server_address(&self) -> String {
        format!("{}:{}", self.host, self.ws_server.port)
    }
}
