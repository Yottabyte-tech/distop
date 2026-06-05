// Load TUI elements
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, BorderType},
    Frame,
};
use std::io;

// Uses all of the files
mod head;
mod worker;
mod message;
mod handle_data;
// Files in get_data
mod get_data;

// Import the node list var for importing data
use handle_data::NODE_LIST;
use handle_data::NodeInfo;

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
        worker::run(head_ip, args.port).await.expect("");

    } 
    else if args.head {

        // Spin up the TUI task and the head logic at the same time
        let head_logic = tokio::spawn(head::run(args.port));
        let tui_logic = tokio::spawn(render_tui());

        // Get if one fails, mostly if the user presses "q"
        tokio::select! {
            res = head_logic => {
                if res.is_ok() { println!(""); }
            }
            res = tui_logic => {
                if res.is_ok() { println!("Quit"); }
            }
        }

    } else {
        println!("Please run with either --head or --worker");
    }
    Ok(())
}

async fn render_tui() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(render_app)?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    ratatui::restore();

    Ok(())
}

fn render_app(frame: &mut Frame) {

    // Import the data list
    let mut list = NODE_LIST.lock().unwrap();
   
    if( list.len() > 0 ){
    

        // Split the terminal horizontally into 2 equal columns (50% each)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
    
        // Create the first paragraph block
        let block1 = Paragraph::new(list[0].info.cpu_usage.to_string())
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title("╯CPU╰")
                   .border_type(BorderType::Rounded)
            );
    
        // Create the second paragraph block
        let block2 = Paragraph::new("Hello World 2!")
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title("╯RAM╰")
                   .border_type(BorderType::Rounded)
            );
    
        // Render them in the split chunks
        frame.render_widget(block1, chunks[0]);
        frame.render_widget(block2, chunks[1]);
    }
}
