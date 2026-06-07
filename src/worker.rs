// Imports the WorkerInfo struct that is sent to the head
use crate::message::WorkerInfo;
// Imports TCP socket connection as an async, used to establish connection
use tokio::net::TcpStream;
// Imports write_all() method to send data asynchronously
use tokio::io::AsyncWriteExt;
// Represents time durations
use std::time::Duration;
// Imports sysinfo to read usage details
use sysinfo::{System, RefreshKind, Components};


use crate::get_data::processes::running_processes;


pub async fn run(head_ip: String, port: u16) -> anyhow::Result<()> {

    // Gets the computer's hostname, otherwise returns "unknown"
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());



    // Initialize the system state ONCE here so it retains metrics history across iterations
    let mut sys = System::new_with_specifics(RefreshKind::everything());
    sys.refresh_all();

    // Main infinite loop
    loop {
        // Tries to connect to the head node
        match TcpStream::connect(format!("{}:{}", head_ip, port)).await {

            Ok(mut stream) => {
                // Pass the mutable reference into the sender loop
                while let Ok(_) = send_worker_info(&mut stream, &hostname, &mut sys).await {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
            Err(_e) => {

                // When connection is lost to head node, retry connection every 1 second
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// Async data send function
async fn send_worker_info(stream: &mut TcpStream, hostname: &str, sys: &mut System) -> Result<(), std::io::Error> {
    // Reads all data and pushes it to sys (tracks true delta since the last tick)
    sys.refresh_all();
    

    let mut cpu_temp: String = String::default();

    let components = Components::new_with_refreshed_list();

    for component in &components {
        let label = component.label().to_lowercase();
        
        if label.contains("cpu") {
            if let Some(temp) = component.temperature() {
                // REMOVED "Some()" from the text template here:
                cpu_temp = format!("{:.1}°C", temp);

                println!("{}", cpu_temp);
            }
        }
    }

    let ram_used = sys.used_memory() as f64 / 1_073_741_824.0; // Gets RAM usage
    let ram_total = sys.total_memory() as f64 / 1_073_741_824.0; // Gets ammount of system RAM

    let mut cores_list = Vec::new();
    for cpu in sys.cpus().iter() {
        cores_list.push(cpu.cpu_usage());
    }

    // Adds all of the data to the message.rs WorkerInfo struct
    let info = WorkerInfo {
        hostname: hostname.to_string(),
        cpu_usage: sys.global_cpu_usage(),
        ram_used_gb: ram_used,
        ram_total_gb: ram_total,
        process_count: sys.processes().len(),
        processes: running_processes(),
        cores: cores_list,
        cpu_temp: cpu_temp,
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

