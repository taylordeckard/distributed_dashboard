# distributed dashboard

## Getting Started

### Prerequisites
- [rust](https://www.rust-lang.org/) installed

### Run the Hub
When running this program in "hub" mode, an http/websocket server listens for and manages connections.

Run the hub with:
```
cargo run -- hub
```

### Run the Client

Running this program in "client" mode will run several threads that serve different purposes:

1. A Websocket client that connects to a hub running elsewhere.
2. A cpu monitoring process that saves metrics to sqlite at a scheduled interval. 
3. A cleanup task that deletes rows from sqlite after a defined expiration date.

Run the client with:
```
cargo run -- client
```

## Development

The following commands can be used to clean and format the code in this repo.
```
cargo fmt
```
```
cargo clippy --fix --allow-dirty -- -W clippy::pedantic
```
