use crate::db::expire_records;
use crate::db::EXPIRE_SECONDS;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time;

pub async fn run(running: Arc<AtomicBool>) {
    let mut interval = time::interval(Duration::from_secs(EXPIRE_SECONDS));

    interval.tick().await;

    while running.load(Ordering::SeqCst) {
        // Wait for the next interval or until interrupted
        tokio::select! {
            _ = interval.tick() => {},
            () = async { while running.load(Ordering::SeqCst) { time::sleep(Duration::from_secs(1)).await; } } => {
                break;
            }
        }

        // Insert CPU usage into the database
        if let Err(e) = expire_records() {
            eprintln!("Error expiring records: {e}");
        }
    }

    println!("Cleanup loop exiting");
}
