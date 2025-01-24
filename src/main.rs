use sysinfo::{System, ProcessesToUpdate};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration};
use termion::{clear, cursor};
use users::{get_user_by_uid};

fn get_process_user(pid: i32) -> String {
    let status_path = format!("/proc/{}/status", pid);
    match fs::read_to_string(status_path) {
        Ok(content) => {
            if let Some(uid_line) = content.lines().find(|line| line.starts_with("Uid:")) {
                if let Some(uid_str) = uid_line.split_whitespace().nth(1) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        if let Some(user) = get_user_by_uid(uid) {
                            return user.name().to_string_lossy().to_string();
                        }
                    }
                }
            }
            "Unknown".to_string()
        },
        Err(_) => "Unknown".to_string(),
    }
}

fn main() {
    // Configurable interval and process limit
    let update_interval = Duration::from_secs(3);
    let process_limit = 10;  // Change this to display more or fewer processes

    // HashMap to store average values for processes
    let mut process_data: HashMap<i32, (f64, f64, u64)> = HashMap::new();

    // Initialize sysinfo to collect system data
    let mut system = System::new_all();

    // Display app name and empty line at the beginning
    print!("{}{}", clear::All, cursor::Goto(1, 1));
    println!("collete v.1.0 by dh (c) 2025");
    println!();

    // Print headers once at the top
    println!(
        "{:<20} {:<15} {:<15} {:<15} {:<15}",
        "Process Name", "User Name", "CPU Usage", "RAM Usage", "Avg CPU Usage"
    );

    // Initial refresh of processes
    system.refresh_processes(ProcessesToUpdate::All, true);

    // Main loop for updating processes
    loop {
        // Refresh processes data
        system.refresh_processes(ProcessesToUpdate::All, true);

        // Get processes sorted by CPU usage in descending order
        let mut processes: Vec<_> = system.processes().into_iter().collect();
        processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());

        // Limit to the top processes
        let top_processes = &processes[..std::cmp::min(process_limit, processes.len())];

        // Iterate over the top processes and update the display
        for (row, (pid, process)) in top_processes.iter().enumerate() {
            let name = process.name().to_string_lossy().to_string();
            let user = get_process_user(pid.as_u32() as i32);
            let cpu_usage = process.cpu_usage();
            let memory_usage = process.memory() as f64 / (1024.0 * 1024.0);

            // Track average CPU and RAM usage for the process over time
            let entry = process_data.entry(pid.as_u32() as i32).or_insert((0.0, 0.0, 0));
            let (avg_cpu, avg_ram, count) = entry;
            let total_cpu = *avg_cpu * (*count as f64) + cpu_usage as f64;
            let total_ram = *avg_ram * (*count as f64) + memory_usage;
            *count += 1;
            *avg_cpu = total_cpu / (*count as f64);
            *avg_ram = total_ram / (*count as f64);

            // Move to the correct row and overwrite the previous line
            print!(
                "{}{:<20} {:<15} {:<15.2} {:<15.2} {:<15.2}",
                cursor::Goto(1, (row + 3).try_into().unwrap()),
                name,
                user,
                cpu_usage,
                memory_usage,
                *avg_cpu
            );
        }

        // If there are fewer processes than the limit, print empty lines
        for row in top_processes.len()..process_limit {
            print!(
                "{}{:<20} {:<15} {:<15} {:<15} {:<15}",
                cursor::Goto(1, (row + 3).try_into().unwrap()),
                "", "", "", "", ""
            );
        }

        // Wait for the next update interval
        std::thread::sleep(update_interval);
    }
}
