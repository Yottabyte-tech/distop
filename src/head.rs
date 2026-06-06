// Imports the info struct from message.rs
use crate::message::WorkerInfo;
// Async to listen for incoming connections
use tokio::net::TcpListener;
// Imports for stream handling and line decoding
use futures_util::StreamExt;
// Decoder trait must be in scope to use the .framed() method
use tokio_util::codec::{Decoder, LinesCodec};


use crate::handle_data::process_data;
use crate::handle_data::create_node;
use crate::handle_data::remove_node;


pub async fn run(port: u16) -> anyhow::Result<()> {
    
    // Creates the TCP listener and awaits for the binding to complete
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    // Main loop
    loop {
        // Waits for new worker to connect to and returns the connection socket and ip/port in addr
        let (socket, addr) = listener.accept().await?;
        
        // Starts a new Async to read from each worker individually
        tokio::spawn(async move {
            // Converts the socket into a Stream of text lines using .framed()
            // This automatically handles chunking and grows internal buffers beyond 8KB if needed
            let mut reader = LinesCodec::new().framed(socket);
            let mut first_time: bool = true;

            let mut node_name: String = "".to_string();


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
                                if first_time{
                                    create_node(&info);
                                    first_time = false;
                                }
                                node_name = info.clone().hostname;
                                process_data(&info, &node_name);
                            }
                            Err(e) => {
                                eprintln!("[-] Failed to parse WorkerInfo from line: {}", e);
                                remove_node(&node_name);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[-] Stream error or unexpected disconnect from {}: {}", addr, e);
                        remove_node(&node_name);
                        break;
                    }
                }
            }
            remove_node(&node_name);
        });
    }
}
