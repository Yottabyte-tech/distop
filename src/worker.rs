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
use std::fs;
use std::path::Path;


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

    match get_cpu_temp() {
        Some(temp) => cpu_temp = format!("{:.1}°C", temp),
        None => cpu_temp = format!(""),
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





pub fn get_cpu_temp() -> Option<f64> {
    if let Ok(hwmon_entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in hwmon_entries.flatten() {
            if let Ok(canonical) = fs::canonicalize(entry.path()) {
                if let Ok(files) = fs::read_dir(&canonical) {
                    for file in files.flatten() {
                        let filename = file.file_name().to_string_lossy().into_owned();
                        if filename.starts_with("temp") && filename.ends_with("_input") {
                            let base = file.path().to_string_lossy().replace("input", "");
                            let label = fs::read_to_string(format!("{}label", base)).unwrap_or_default();
                            
                            if label.starts_with("Package id") || label.starts_with("Tdie") || label.starts_with("SoC Temperature") {
                                if let Ok(raw) = fs::read_to_string(file.path()) {
                                    if let Ok(temp_val) = raw.trim().parse::<f64>() {
                                        return Some(temp_val / 1000.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut i = 0;
    loop {
        let zone_path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if !Path::new(&zone_path).exists() {
            break;
        }
        if let Ok(raw) = fs::read_to_string(&zone_path) {
            if let Ok(temp_val) = raw.trim().parse::<f64>() {
                return Some(temp_val / 1000.0);
            }
        }
        i += 1;
    }

    None
}
