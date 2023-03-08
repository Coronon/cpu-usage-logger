use chrono::Local;
use clap::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use std::{collections::HashMap, thread};
use sysinfo::{Pid, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

/// Simple utility to log high CPU usage
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// How long to wait between measurements in seconds
    #[arg(short = 'b', long, default_value_t = 5)]
    time_between_measurements: u64,

    /// How long to measure for in seconds (CPU usage is an average over this time)
    #[arg(short, long, default_value_t = 1)]
    measurement_time: u64,

    /// Threshold of total CPU usage to start logging at in percent
    #[arg(short, long, default_value_t = 30.0)]
    total_log_threshold: f32,

    /// Threshold of single process CPU usage to start logging at in percent
    #[arg(short, long, default_value_t = 15.0)]
    process_log_threshold: f32,

    /// Number of top CPU consuming processes to log when `total_log_threshold` is exceeded and to show in the CLI
    #[arg(short, long, default_value_t = 5)]
    number_of_processes_to_show: usize,

    /// CLI mode -> periodically write stats to stdout
    #[arg(short, long, default_value_t = false)]
    cli: bool,

    /// Path to log file
    #[arg(short, long)]
    log_file: Option<String>,
}

/// CPU usage stats for a process
struct ProcessStats<'a> {
    got_cpu_usage: f32,
    process: &'a Process,
}

impl<'a> From<&'a Process> for ProcessStats<'a> {
    fn from(prcs: &'a Process) -> Self {
        ProcessStats {
            got_cpu_usage: 0.0,
            process: prcs,
        }
    }
}

/// CPU usage stats for the whole system (including processes)
struct CPUStats<'a> {
    processes: Vec<ProcessStats<'a>>,
}

/// Convert `sys.processes()` to [CPUStats]
impl<'a> From<&'a HashMap<Pid, Process>> for CPUStats<'a> {
    fn from(value: &'a HashMap<Pid, Process>) -> Self {
        CPUStats {
            processes: value
                .values()
                .map(|v| ProcessStats {
                    got_cpu_usage: 0.0,
                    process: v,
                })
                .collect::<Vec<ProcessStats>>(),
        }
    }
}

fn main() {
    //* Parse args
    let args = Args::parse();

    //* Process
    // Init process tracking
    let proc_refresh_kind = ProcessRefreshKind::new().with_cpu();
    let mut sys = System::new_all();
    let cpu_count = sys.physical_core_count().unwrap() as f32;

    loop {
        // Refresh CPU
        sys.refresh_processes_specifics(proc_refresh_kind);

        // Get currently running processes
        let mut cpu_stats: CPUStats = CPUStats::from(sys.processes());

        // Start CPU calculation
        for p_info in &cpu_stats.processes {
            p_info.process.cpu_usage();
        }

        // Wait to collect data between time points
        thread::sleep(Duration::from_secs(args.measurement_time));

        // Update CPU usage
        cpu_stats
            .processes
            .iter_mut()
            .for_each(|p| p.got_cpu_usage = p.process.cpu_usage() / cpu_count);

        // Sort by usage
        cpu_stats.processes.sort_by(|a, b| {
            a.got_cpu_usage
                .partial_cmp(&b.got_cpu_usage)
                .unwrap()
                .reverse()
        });

        // Calculate total usage by all processes
        let total_cpu_usage: f32 = cpu_stats.processes.iter().map(|v| v.got_cpu_usage).sum();
        let mut formatted_stats: Option<String> = None;

        //* Handle thresholds
        let mut total_cpu_usage_message: Option<String> = None;
        if total_cpu_usage >= args.total_log_threshold {
            // We always have to format the stats here
            formatted_stats = Some(format_stats(
                &cpu_stats,
                total_cpu_usage,
                args.number_of_processes_to_show,
            ));

            total_cpu_usage_message = Some(format!(
                "Total CPU usage threshold of {:.2}% exceeded -> {:.2}%",
                args.total_log_threshold, total_cpu_usage,
            ));

            // If we would push the whole logged message into total_cpu_usage_message the
            // CLI would display the usage table twice
            let logged_message = format!(
                "{}\n{}",
                total_cpu_usage_message.as_ref().unwrap(),
                formatted_stats.as_ref().unwrap(),
            );

            log_to_file(&args.log_file, &logged_message);
        }

        let mut process_cpu_usage_message: Option<String> = None;
        cpu_stats
            .processes
            .iter()
            .take_while(|p| p.got_cpu_usage >= args.process_log_threshold)
            .for_each(|p| {
                let existing_string = match process_cpu_usage_message.as_ref() {
                    Some(s) => format!("{}\n", s),
                    None => String::new(),
                };

                process_cpu_usage_message = Some(format!(
                    "{}Single process CPU usage threshold of {:.2}% exceeded -> [Pid: {}] Name: '{}' Usage: {:.2}%",
                    existing_string,
                    args.process_log_threshold,
                    p.process.pid(),
                    p.process.name(),
                    p.got_cpu_usage,
                ));
            });
        if process_cpu_usage_message.is_some() {
            log_to_file(&args.log_file, process_cpu_usage_message.as_ref().unwrap());
        }

        //* Print results
        if args.cli {
            // Ensure we only format stats if needed
            formatted_stats = formatted_stats.or_else(|| {
                Some(format_stats(
                    &cpu_stats,
                    total_cpu_usage,
                    args.number_of_processes_to_show,
                ))
            });

            // Clear old output (we use a library because different consoles /os's support different ways of clearing)
            clearscreen::clear().expect("failed to clear screen");

            // Write new output
            println!("{}", formatted_stats.as_ref().unwrap());

            if total_cpu_usage_message.is_some() {
                println!("\n{}", total_cpu_usage_message.as_ref().unwrap());
            }

            if process_cpu_usage_message.is_some() {
                println!("\n{}", process_cpu_usage_message.as_ref().unwrap());
            }
        }

        // Wait for next iteration
        thread::sleep(Duration::from_secs(args.time_between_measurements));
    }
}

/// Formats stats into a nice looking table
fn format_stats(cpu_stats: &CPUStats, total_cpu_usage: f32, num_processes: usize) -> String {
    format!(
        "{header}\n{total_cpu_usage}\n{timestamp}\n{divider}\n{column_names}\n{column_names_divider}\n{stats}\n{divider}",
        header = format_args!("{:-^80}", "CPU usage"),
        total_cpu_usage = format_args!("|{: ^78}|", format!("{:.2} %", total_cpu_usage)),
        timestamp = format_args!("|{: ^78}|", get_iso_time()),
        divider = format_args!("{:-^80}", ""),
        column_names = format_args!("| {0: <10} | {1: <50} | {2: <10} |", "PID", "Name", "Usage"),
        column_names_divider = format_args!("|{0:-<12}|{1:-<52}|{2:-<12}|", "", "", ""),
        stats = cpu_stats.processes.iter().take(num_processes).map(|p| {
            format!(
                "| {0: <10} | {1: <50} | {2: <10} |",
                p.process.pid().to_string(),
                p.process.name().to_string(),
                format!("{:.2} %", p.got_cpu_usage),
            )
        }).fold(String::new(), |ret, new| format!("{}\n{}", ret, new)).trim()
    )
}

/// Log a message to a file with timestamp, ending in a new line
fn log_to_file(file_path: &Option<String>, message: &str) {
    // Don't log anything if no path specified
    if file_path.is_none() {
        return;
    }

    // Prepend ISO timestamp to every line
    let pre_text = format!("{} | ", get_iso_time());
    let processed_message = format!("\n{}\n", message)
        .split('\n')
        .map(|m| format!("{}{}", pre_text, m))
        .collect::<Vec<String>>()
        .join("\n");

    // Open file in append mode
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path.as_ref().unwrap())
        .unwrap();

    writeln!(file, "{}", processed_message).unwrap();
}

/// Get current DateTime as an ISO 8601 formatted string
fn get_iso_time() -> String {
    // 2023-03-08T21:19:47.101382300+01:00
    format!("{}", Local::now().format("%Y-%m-%dT%H:%M:%S%.9f%:z"))
}
