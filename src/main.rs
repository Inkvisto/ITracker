mod args;
mod config;
mod csv_export;
mod log;
mod timer;
mod tui;

use args::Args;
use clap::{error::ErrorKind, Parser};
use config::{load_config, save_config};
use log::read_logs_from_file;
use std::time::SystemTime;
use timer::{log_elapsed_time, Timer};

/// Main function to execute the program logic.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = parse_args();

    // Handle log deletion if specified
    if let Some(index) = args.delete_log {
        let log_file = args.log.as_deref().unwrap_or("logs.txt");
        log::delete_log_entry(log_file, index)?;
        println!("Log entry at index {} deleted from {}.", index, log_file);
        return Ok(());
    }

    // Read logs from the specified file if provided
    let logs = if let Some(ref log_file) = args.log {
        read_logs_from_file(log_file)?
    } else {
        vec![]
    };

    // Render TUI if necessary and capture title and description
    let data = if args.log.is_some() {
        tui::render(Some(logs))?
    } else if args.add {
        tui::render(None)?
    } else {
        vec![String::new()]
    }
    .join("");

    // Load or save configuration
    let output_file = manage_config(&args)?;

    println!("Using output file: {}", output_file);

    // Handle timer commands like start, pause, resume, and stop
    handle_timer_commands(args, data, &output_file)?;

    Ok(())
}

/// Helper function to parse command line arguments.
fn parse_args() -> Args {
    Args::try_parse().unwrap_or_else(|err| {
        if err.kind() == ErrorKind::DisplayHelp || err.kind() == ErrorKind::DisplayVersion {
            // Print help or version information
            eprintln!("{}", err);
        } else {
            // Optionally, you can print a custom error message
            eprintln!("Error parsing arguments: {}", err);
        }
        // Return a default instance of Args
        Args::default() // Ensure Args implements Default trait or return an appropriate fallback
    })
}

/// Helper function to manage configuration loading/saving.
fn manage_config(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = load_config()?;

    let output_file = if let Some(ref file) = args.output_file {
        let file_str = file.to_string_lossy().into_owned();
        config.output_file = Some(file_str.clone());
        save_config(&config)?;
        file_str
    } else {
        config.output_file.clone().unwrap_or_default()
    };

    Ok(output_file)
}

/// Handle timer commands like start, pause, resume, and stop.
fn handle_timer_commands(
    args: Args,
    data: String,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut timer = Timer::new();

    if args.start {
        start_timer(&mut timer);
    }
    if args.pause {
        pause_timer(&mut timer);
    }
    if args.resume {
        resume_timer(&mut timer);
    }
    if !data.is_empty() {
        log_task_details(&mut timer, &data, output_file)?;
    }
    if args.stop {
        stop_timer(&mut timer, output_file)?;
    }

    // Log daily time and save it to a file after stopping the timer
    timer.log_daily_time()?;
    timer.save_daily_log(output_file)?;

    // Optionally, show summaries if needed
    if args.show_daily_summary {
        print_summary("Daily Summary", timer.get_daily_summary());
    }
    if args.show_weekly_summary {
        print_summary("Weekly Summary", timer.get_weekly_summary());
    }

    // Export logs to CSV if requested
    if args.export_to_csv {
        timer.export_logs_to_csv(output_file)?;
    }

    Ok(())
}

/// Start the timer.
fn start_timer(timer: &mut Timer) {
    timer.start();
    println!("Timer started.");
}

/// Pause the timer.
fn pause_timer(timer: &mut Timer) {
    timer.pause();
    println!("Timer paused.");
}

/// Resume the timer.
fn resume_timer(timer: &mut Timer) {
    timer.resume();
    println!("Timer resumed.");
}

/// Log task details to the timer.
fn log_task_details(
    timer: &mut Timer,
    data: &str,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    timer.log_task(data, output_file, SystemTime::now())?;
    Ok(())
}

/// Stop the timer and log the elapsed time.
fn stop_timer(timer: &mut Timer, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    timer.stop();
    let stopped_time = timer.stopped_time().unwrap();
    let elapsed_time = stopped_time
        .duration_since(timer.started_time().unwrap())
        .unwrap_or_default();

    println!(
        "Timer stopped at {:?}. Elapsed time: {:?}",
        stopped_time, elapsed_time
    );

    // Log the elapsed time
    log_elapsed_time(elapsed_time, output_file)?;

    Ok(())
}

/// Function to print summaries.
fn print_summary(
    title: &str,
    summary: std::collections::HashMap<chrono::NaiveDate, std::time::Duration>,
) {
    println!("\n{}:", title);
    for (date, duration) in summary {
        println!(
            "Date: {}, Time Logged: {} seconds",
            date,
            duration.as_secs()
        );
    }
}
