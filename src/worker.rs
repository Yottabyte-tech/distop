// Imports the WorkerInfo struct that is sent to the head
use crate::message::WorkerInfo;
// Imports TCP socket connection as an async, used to establish connection
use tokio::net::TcpStream;
// Imports write_all() method to send data asynchronously
use tokio::io::AsyncWriteExt;
// Represents time durations
use std::time::Duration;
// Imports sysinfo to read usage details
use sysinfo::{System, RefreshKind};

pub async fn run(head_ip: String, port: u16) -> anyhow::Result<()> {
    println!("Worker started → Connecting to head node {}:{}", head_ip, port);
    
    // Gets the computer's hostname, otherwise returns "unknown"
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Main infinite loop
    loop {
        // Tries to connect to the head node
        match TcpStream::connect(format!("{}:{}", head_ip, port)).await {

            Ok(mut stream) => {
                println!("Connected to head node. Sending updates...");
                // Actually sends data and loops every 3 seconds
                while let Ok(_) = send_worker_info(&mut stream, &hostname).await {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
            Err(e) => {

                // When connection is lost to head node, retry connection every 5 seconds
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// Async data send function
async fn send_worker_info(stream: &mut TcpStream, hostname: &str) -> Result<(), std::io::Error> {
    // New info collector to read system data
    let mut sys = System::new_with_specifics(RefreshKind::everything());

    // Reads all data and pushes it to sys
    sys.refresh_all();

    let ram_used = sys.used_memory() as f64 / 1_073_741_824.0; // Gets RAM usage
    let ram_total = sys.total_memory() as f64 / 1_073_741_824.0; // Gets ammount of system RAM
    let load = System::load_average(); // Gets the system load

    // Adds all of the data to the message.rs WorkerInfo struct
    let info = WorkerInfo {
        hostname: hostname.to_string(),
        timestamp: chrono::Utc::now(),
        cpu_usage: sys.global_cpu_usage(),
        ram_used_gb: ram_used,
        ram_total_gb: ram_total,
        load_average_1min: load.one,
        process_count: sys.processes().len(),
        status: "online".to_string(),
    };
    
    // Converts struct to JSON
    let mut json = serde_json::to_string(&info).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    // Adds a newline to act as a delimiter
    json.push('\n');

    // Sends the compiled JSON to the head
    stream.write_all(json.as_bytes()).await?;
    
    // Confirms success
    Ok(())
}
