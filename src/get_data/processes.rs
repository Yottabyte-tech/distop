use procfs::process;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub virtual_memory_bytes: u64,
    pub resident_set_size_bytes: u64,
    pub cpu_usage_percentage: f32,
}

pub struct HumanBytes(pub u64);

impl std::fmt::Display for HumanBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0 as f64;
        if bytes >= 1_073_741_824.0 {
            write!(f, "{:.2} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            write!(f, "{:.2} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            write!(f, "{:.2} KB", bytes / 1024.0)
        } else {
            write!(f, "{} B", self.0)
        }
    }
}

fn get_process_ticks(proc: &process::Process) -> u64 {
    let mut total_ticks = 0;
    if let Ok(tasks) = proc.tasks() {
        for task_res in tasks.flatten() {
            if let Ok(stat) = task_res.stat() {
                total_ticks += stat.utime + stat.stime;
            }
        }
    }
    if total_ticks == 0 {
        if let Ok(stat) = proc.stat() {
            total_ticks = stat.utime + stat.stime;
        }
    }
    total_ticks
}

pub fn get_running_processes() -> Vec<ProcessInfo> {
    let mut process_list = Vec::new();
    
    let num_cpus = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1) as f32;

    let ticks_per_second = procfs::ticks_per_second() as f32;

    let mut process_baseline = HashMap::new();
    if let Ok(all_procs) = process::all_processes() {
        for proc_res in all_procs.flatten() {
            let ticks = get_process_ticks(&proc_res);
            process_baseline.insert(proc_res.pid, ticks);
        }
    }
    
    let start_time = Instant::now();

    thread::sleep(Duration::from_millis(500));

    let elapsed_seconds = start_time.elapsed().as_secs_f32();

    if let Ok(all_procs) = process::all_processes() {
        for proc_res in all_procs.flatten() {
            if let Ok(stat) = proc_res.stat() {
                let current_proc_ticks = get_process_ticks(&proc_res);
                let previous_proc_ticks = *process_baseline.get(&proc_res.pid).unwrap_or(&current_proc_ticks);
                let proc_delta_ticks = current_proc_ticks.saturating_sub(previous_proc_ticks) as f32;

                let cpu_seconds_used = proc_delta_ticks / ticks_per_second;
                let cpu_percentage = (cpu_seconds_used / elapsed_seconds) / num_cpus * 100.0;

                process_list.push(ProcessInfo {
                    pid: proc_res.pid,
                    name: stat.comm,
                    virtual_memory_bytes: stat.vsize,
                    resident_set_size_bytes: stat.rss * 4096, 
                    cpu_usage_percentage: cpu_percentage,
                });
            }
        }
    }

    process_list
}

pub fn running_processes() -> Vec<ProcessInfo> {
    let mut processes = get_running_processes();

    processes.sort_by(|a, b| {
        b.cpu_usage_percentage
            .partial_cmp(&a.cpu_usage_percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.resident_set_size_bytes.cmp(&a.resident_set_size_bytes))
    });

    println!(
        "{:<8} {:<25} {:<10} {:<15} {:<15}",
        "PID", "Name", "CPU (%)", "VIRT", "RSS"
    );
    println!("{:-<75}", "");

    for proc in &processes {
        println!(
            "{:<8} {:<25} {:<10.2} {:<15} {:<15}",
            proc.pid,
            proc.name,
            proc.cpu_usage_percentage,
            HumanBytes(proc.virtual_memory_bytes).to_string(),
            HumanBytes(proc.resident_set_size_bytes).to_string()
        );
    
    }


    // let new_len = processes.len().saturating_sub(processes.len().saturating_sub(10));
    // processes.truncate(new_len);

    return processes;
}

