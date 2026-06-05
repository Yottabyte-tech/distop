// Uses all of the files
mod head;
mod worker;
mod message;

// Files in get_data
mod get_data;
use crate::get_data::processes::running_processes;
// Parser for the command line that reads flags and args
use clap::Parser;

// Macro that generates code to parse CLI args
#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long)]
    head: bool,

    #[arg(long)]
    worker: bool,

    #[arg(long)]
    ip: Option<String>,
}

// Macro that turns the main function into an async
#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Parses args into the args struct
    let args = Args::parse();
    
    // Decides what type of node this computer is
    if args.worker {
        let head_ip = args.ip.expect("--ip <HEAD_IP> is required when running as worker");
        worker::run(head_ip, args.port).await
    } else if args.head {
        head::run(args.port).await
    } else {
        println!("Please run with either --head or --worker");
        Ok(())
    }
}
