use crate::db::insert_cpu_usage;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use sysinfo::System;
use tokio::time;

const CPU_CHECK_WAIT: u64 = 5;

pub async fn cpu_monitoring_loop(running: Arc<AtomicBool>) {
    let mut sys = System::new_all();
    let mut interval = time::interval(Duration::from_secs(CPU_CHECK_WAIT));

    interval.tick().await;

    while running.load(Ordering::SeqCst) {
        // Refresh CPU data
        sys.refresh_cpu_all();

        // Get CPU usage
        let cpu_usage = sys.global_cpu_usage();

        println!("CPU Usage: {cpu_usage:.2}%");

        // Insert CPU usage into the database
        if let Err(e) = insert_cpu_usage(cpu_usage) {
            eprintln!("Error inserting CPU usage: {e}");
        }

        // Wait for the next interval or until interrupted
        tokio::select! {
            _ = interval.tick() => {},
            () = async { while running.load(Ordering::SeqCst) { time::sleep(Duration::from_secs(1)).await; } } => {
                break;
            }
        }
    }

    println!("CPU monitoring loop exiting");
}
