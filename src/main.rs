// Load TUI elements
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, BorderType},
    Frame,
};
use ratatui::style::Color;
use ratatui_macros::text;
use ratatui::prelude::{Text, Line, Span, Stylize};
use std::sync::{LazyLock, Mutex};

// Uses all of the files
mod head;
mod worker;
mod message;
mod handle_data; // Files in get_data
mod get_data;

// Import the node list var for importing data
use handle_data::NODE_LIST;

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
    include: bool,
    
    #[arg(long)]
    ip: Option<String>,
}


pub struct CoreGraph {
    pub hostname: String,
    pub core_index: i32,
    pub graph_data: String,
}
pub struct RAMGraph {
    pub hostname: String,
    pub graph_data: String,
}


pub static NODE_INDEX: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

pub static CORE_GRAPHS: LazyLock<Mutex<Vec<CoreGraph>>> = LazyLock::new(|| Mutex::new(Vec::new()));
pub static RAM_GRAPHS: LazyLock<Mutex<Vec<RAMGraph>>> = LazyLock::new(|| Mutex::new(Vec::new()));


// Macro that turns the main function into an async #[tokio::main]
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
        
        if args.include {
            
            let port = args.port;

            tokio::spawn(worker::run("127.0.0.1".to_string(), port));

        }
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
        if event::poll(std::time::Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
                if key.code == KeyCode::Up {
                    let mut index = NODE_INDEX.lock().unwrap();
                    if *index == 0 {
                        *index = NODE_LIST.lock().unwrap().len() as i32 - 1;
                    } else {
                        *index -= 1;
                    }
                }
                if key.code == KeyCode::Down {
                    let mut index = NODE_INDEX.lock().unwrap();
                    if *index == NODE_LIST.lock().unwrap().len() as i32 - 1{
                        *index = 0;
                    } else {
                        *index += 1;
                    }
                }
            }
        }
    }

    ratatui::restore();

    Ok(())
}

fn render_app(frame: &mut Frame) {

    // Import the data list
    let list = NODE_LIST.lock().unwrap();
    if list.len() > 0 {

        let mut name_select = Text::default();

        let node_index: i32 = *NODE_INDEX.lock().unwrap() as i32;
        
        for (index, node) in list.iter().enumerate(){
             
            let hostname: &String = &node.info.hostname;

            if node_index == index as i32{
                name_select.lines.push(Line::from(format!("{}", hostname)).black().on_white());
            } else {
                name_select.lines.push(Line::from(format!("{}", hostname)));
            }
        }
        
        let graph_chars: Vec<String> = vec![
            "⣀".to_string(),
            "⣤".to_string(),
            "⣶".to_string(),
            "⣿".to_string(),
        ];
        



        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(32), Constraint::Min(52), Constraint::Min(42), Constraint::Percentage(100)])
            .split(frame.area());


        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(horizontal[3]);

        
        let mut computer_usage = Text::default();
        
        let mut ram_usage = Text::default();

        let mut network_usage = Text::default();

        

        let list_elem = &list[node_index as usize];






        let node_cpu_usage = format!("CPU: [ {:.2}% ]", list_elem.info.cpu_usage);
            
        // Push each line explicitly to allow granular styling
        computer_usage.lines.push(Line::from("╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌").fg(Color::Rgb(100,100,100))); // Second newline
        computer_usage.lines.push(Line::from(vec![
            Span::raw("╭─┤ "),
            Span::raw(list_elem.info.hostname.to_string()).bold().underlined(),
            Span::raw(" │"),
        ]));
        computer_usage.lines.push(Line::from("│"));
        computer_usage.lines.push(Line::from(format!("├─┤ {} {}", node_cpu_usage, list_elem.info.cpu_temp)));
        computer_usage.lines.push(Line::from("│"));
 

        for (_index, core) in list_elem.info.cores.iter().enumerate(){
                
            let core_index: i32 = _index as i32;
                
            let mut core_list = CORE_GRAPHS.lock().unwrap();
                
            let core_graph_data = core_list.iter().position(|item| item.core_index == core_index && item.hostname == list_elem.info.hostname.to_string());
                
            let map_to_bar: f32 = (core/100.0) * 4.0;

            let mut graph: String = graph_chars[map_to_bar.round().max(1.0) as usize - 1].clone();
                
            match core_graph_data {
                Some(_index) => {

                    // Get graphing struct
                    let item = &core_list[_index];
                        
                    let mut new_graph = format!("{}{}", item.graph_data.clone(), graph);
                        
                    if new_graph.chars().count() > 20 {
                        if !new_graph.is_empty() {
                            new_graph.remove(0); 
                        }
                    }

                    core_list[_index].graph_data = new_graph.clone();
                        
                    graph = new_graph;
                }
                None => core_list.push(CoreGraph { hostname: list_elem.info.hostname.to_string(), core_index: core_index, graph_data: "                    ".to_string()}),
            }

            // Graph spacing
            let spacing: usize = 10 - format!("{}{:.2}%", core_index, core).len().min(10);
                
            let core_u8: u8 = *core as u8;
                
            let mut connecting_char: String = "├".to_string();

            if core_index + 1 as i32 == list_elem.info.cores.len() as i32{
                connecting_char = "╰".to_string();
            }

            let core_line = Line::from(vec![
                Span::raw(format!("{}─┤ Core {}: {:.2}%{}[ ", connecting_char, core_index, core, " ".to_string().repeat(spacing))),
                Span::raw(format!("{}", graph)).bold().fg(Color::Rgb(2 * core_u8, 2 * (101 - core_u8), 0)),
                Span::raw(" ]")
            ]);
            computer_usage.lines.push(core_line);
        }


        let mut ram_list = RAM_GRAPHS.lock().unwrap();
            
        let ram_graph_data = ram_list.iter().position(|item| item.hostname == list_elem.info.hostname.to_string());

        let map_to_bar: f64 = (list_elem.info.ram_used_gb/list_elem.info.ram_total_gb) * 4.0;

        let mut graph: String = graph_chars[map_to_bar.round().max(1.0) as usize - 1].clone();
        match ram_graph_data {
            Some(_index) => {

                // Get graphing struct
                let item = &ram_list[_index];
                        
                let mut new_graph = format!("{}{}", item.graph_data.clone(), graph);
                        
                if new_graph.chars().count() > 20 {
                    if !new_graph.is_empty() {
                        new_graph.remove(0); 
                    }
                }

                ram_list[_index].graph_data = new_graph.clone();
                        
                graph = new_graph;
            }
            None => ram_list.push(RAMGraph { hostname: list_elem.info.hostname.to_string(), graph_data: "                    ".to_string()}),
        }
           

        let ram_u8: u8 = (100.0 * list_elem.info.ram_used_gb/list_elem.info.ram_total_gb) as u8;

        ram_usage.lines.push(Line::from("╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌").fg(Color::Rgb(100,100,100))); // Second newline
        ram_usage.lines.push(Line::from(vec![
            Span::raw("╭─┤ "),
            Span::raw(list_elem.info.hostname.clone()).underlined(),
            Span::raw(" │"),
        ]));
        ram_usage.lines.push(Line::from("│"));
        ram_usage.lines.push(Line::from(format!("├─┤ RAM: {:.2}% ({:.2}GB/{:.2}GB)", 100.0 * list_elem.info.ram_used_gb / list_elem.info.ram_total_gb ,list_elem.info.ram_used_gb, list_elem.info.ram_total_gb)));
        ram_usage.lines.push(Line::from("│"));

        ram_usage.lines.push(Line::from(vec![
            Span::raw("╰─┤ [ "),
            Span::raw(graph).bold().fg(Color::Rgb(2 * ram_u8, 2 * (101 - ram_u8), 0)),
            Span::raw(" ]"),
            Span::raw(list_elem.info.network[0].interface.clone()),
        ]));

            
        network_usage.lines.push(Line::from("╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌").fg(Color::Rgb(100,100,100))); // Second newline
        network_usage.lines.push(Line::from(vec![
            Span::raw("╭─┤ "),
            Span::raw(list_elem.info.hostname.clone()).underlined(),
            Span::raw(" │"),
        ]));
        network_usage.lines.push(Line::from("│"));

    
        

        let name_select_block = Paragraph::new(name_select).fg(Color::White)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title(Line::from(vec![
                        Span::raw("╯Nodes╰"),
                   ]))
                   .border_type(BorderType::Rounded)
                   .fg(Color::Rgb(150,75,75))
                   .bg(Color::Rgb(0,0,0))
            );


        // Create the first paragraph block
        let cpu_block = Paragraph::new(computer_usage).fg(Color::White)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title(Line::from(vec![
                        Span::raw("╯CPU╰"),
                   ]))
                   .border_type(BorderType::Rounded)
                   .fg(Color::Rgb(150,150,75))
                   .bg(Color::Rgb(0,0,0))
            );
            
        // Create the second paragraph block
        let ram_block = Paragraph::new(ram_usage).fg(Color::White)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title(Line::from(vec![
                        Span::raw("╯RAM╰"),
                   ]))
                   .border_type(BorderType::Rounded)
                   .fg(Color::Rgb(75,150,75))
                   .bg(Color::Rgb(0,0,0))
            );
            
        let network_block = Paragraph::new(network_usage).fg(Color::White)
            .block(Block::default()
                   .borders(Borders::ALL)
                   .title("╯Nodes╰").white()
                   .border_type(BorderType::Rounded)
                   .fg(Color::Rgb(75,75,150))
                   .bg(Color::Rgb(0,0,0))
            );


        frame.render_widget(name_select_block, horizontal[0]);
        frame.render_widget(cpu_block, horizontal[1]);
        frame.render_widget(ram_block, horizontal[2]);
        frame.render_widget(network_block, vertical[0]);
    }
}

