use clap::{Parser, Subcommand};

/// Distributed Dashboard CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Runs the Client program
    Client {},
    /// Runs the Hub program
    Hub {},
}
