// Imports the info struct from message.rs
use crate::message::WorkerInfo;
// Async to listen for incoming connections
use tokio::net::TcpListener;
// Imports async functions like read() to read sent data
use tokio::io::AsyncReadExt;


pub async fn run(port: u16) -> anyhow::Result<()> {
    
    // Creates the TCP listener and awaits for the binding to complete
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    // Main loop
    loop {
        // Waits for new worker to connect to and returns the connection socket and ip/port in addr
        let (mut socket, addr) = listener.accept().await?;
        println!("[+] Worker connected from {}", addr);
        
        // Starts a new Async to read from each worker individualy
        tokio::spawn(async move {
            // Creates an 8KB buffer to hold the incoming JSON
            let mut buf = vec![0; 8192];
            
            // Creates a new loop that keeps the connection to this worker alive
            loop {
                // Reads the data from the worker and loads it into the buffer,
                // and returns bytes read: n
                match socket.read(&mut buf).await {

                    // Runs if the connection was cleanly closed and breaks out of loop
                    Ok(0) => {
                        println!("[-] Worker {} disconnected", addr);
                        break;
                    }

                    // Reads the first n bytes (The ammount of actual data in the buffer
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]);
                        
                        // Splits the received lines by adding newlines, and skips empty lines
                        for line in data.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            
                            // Parses the recieved JSON into the message.rs struct
                            match serde_json::from_str::<WorkerInfo>(line) {
                                Ok(info) => {
                                    print_worker_info(&addr, &info);
                                }
                                Err(e) => {
                                    eprintln!("JSON parse error from {}: {}", addr, e);
                                }
                            }
                        }
                    }
                    // If the worker won't connect, just forget this worker and break the connection
                    Err(e) => {
                        break;
                    }
                }
            }
        });
    }
}

// Prints all of the data, temporary for debugging
fn print_worker_info(addr: &std::net::SocketAddr, info: &WorkerInfo) {
    println!("{}", "═".repeat(70));   // Fixed
    println!("Host          : {} ({})", info.hostname, addr);
    println!("Time          : {}", info.timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!("CPU Usage     : {:.2}%", info.cpu_usage);
    println!("RAM Usage     : {:.2} / {:.2} GB", info.ram_used_gb, info.ram_total_gb);
    println!("Load Avg (1m) : {:.2}", info.load_average_1min);
    println!("Processes     : {}", info.process_count);
    println!("Status        : {}", info.status);
    println!();
}
