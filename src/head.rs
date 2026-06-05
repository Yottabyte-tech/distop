// Imports the info struct from message.rs
use crate::message::WorkerInfo;
// Async to listen for incoming connections
use tokio::net::TcpListener;
// Imports for stream handling and line decoding
use futures_util::StreamExt;
// Decoder trait must be in scope to use the .framed() method
use tokio_util::codec::{Decoder, LinesCodec};

pub async fn run(port: u16) -> anyhow::Result<()> {
    
    // Creates the TCP listener and awaits for the binding to complete
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    // Main loop
    loop {
        // Waits for new worker to connect to and returns the connection socket and ip/port in addr
        let (socket, addr) = listener.accept().await?;
        println!("[+] Worker connected from {}", addr);
        
        // Starts a new Async to read from each worker individually
        tokio::spawn(async move {
            // Converts the socket into a Stream of text lines using .framed()
            // This automatically handles chunking and grows internal buffers beyond 8KB if needed
            let mut reader = LinesCodec::new().framed(socket);
            
            // Loop that reads complete text lines until the stream ends or encounters an error
            while let Some(result) = reader.next().await {
                match result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        
                        // Parses the received JSON into the message.rs struct
                        match serde_json::from_str::<WorkerInfo>(&line) {
                            Ok(info) => {
                                print_worker_info(&addr, &info);
                            }
                            Err(e) => {
                                eprintln!("[-] Failed to parse WorkerInfo from line: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[-] Stream error or unexpected disconnect from {}: {}", addr, e);
                        break;
                    }
                }
            }
            println!("[-] Worker {} disconnected", addr);
        });
    }
}

// Prints all of the data, temporary for debugging
fn print_worker_info(addr: &std::net::SocketAddr, info: &WorkerInfo) {
    println!("{}", "═".repeat(70));
    println!("Host          : {} ({})", info.hostname, addr);
    println!("Time          : {}", info.timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!("CPU Usage     : {:.2}%", info.cpu_usage);
    println!("RAM Usage     : {:.2} / {:.2} GB", info.ram_used_gb, info.ram_total_gb);
    println!("Load Avg (1m) : {:.2}", info.load_average_1min);
    println!("Processes     : {}", info.process_count);
    println!("Status        : {}", info.status);
    println!("Processes List: {}", info.processes[1].name);
    println!();
}

