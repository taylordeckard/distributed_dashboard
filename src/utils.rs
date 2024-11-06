use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration};

pub async fn wait_for_running_to_be_false(running: Arc<AtomicBool>) {
    while running.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(100)).await;
    }
}
