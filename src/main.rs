mod args;
mod config;
mod log;
mod timer;
mod tui;

use args::Args;
use chrono::{DateTime, Utc};
use clap::{error::ErrorKind as ClapErrorKind, Parser};
use config::{load_config, save_config};
use csv::ReaderBuilder;
use log::read_logs_from_file;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Error, ErrorKind},
    time::{Duration, SystemTime},
};
use timer::{TaskLog, Timer};

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
    handle_commands(args, data, &output_file)?;

    Ok(())
}

fn parse_args() -> Args {
    Args::try_parse().unwrap_or_else(|err| {
        if err.kind() == ClapErrorKind::DisplayHelp || err.kind() == ClapErrorKind::DisplayVersion {
            eprintln!("{}", err);
        } else {
            eprintln!("Error parsing arguments: {}", err);
        }
        Args::default()
    })
}

fn manage_config(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = load_config()?;

    let output_file = if let Some(ref file) = args.output_file {
        let file_str = file.to_string_lossy().into_owned();
        config.output_file = Some(file_str.clone());
        save_config(&config)?;
        file_str
    } else {
        config
            .output_file
            .clone()
            .unwrap_or_else(|| String::from("default_output.txt"))
    };

    Ok(output_file)
}

fn handle_commands(args: Args, data: String, output_file: &str) -> Result<(), std::io::Error> {
    let mut timer = Timer::new();

    if args.add {
        let log_index = start_timer(&mut timer, &data, output_file)?;
        println!("Timer started for log entry at index {}.", log_index);
    }

    if args.pause {
        timer.pause(output_file, 1)?;
    }

    if args.resume {
        timer.resume(output_file, 0)?;
        let elapsed_time = timer.get_elapsed_time(output_file, 1)?;
        println!("Timer paused. Total elapsed time: {:?}", elapsed_time);
    }

    if args.stop.is_some() {
        let index = args
            .stop
            .unwrap_or_else(|| get_last_index_from_csv(output_file).unwrap_or(0));
        stop_timer(&mut timer, output_file, index)?;
    }

    Ok(())
}

fn start_timer(timer: &mut Timer, data: &str, output_file: &str) -> Result<usize, std::io::Error> {
    // Log the task and return the index of the log entry
    timer.log_task(data, output_file)?;

    // Calculate the log index based on the CSV file contents
    let log_index = {
        let mut reader = csv::Reader::from_reader(File::open(output_file)?);
        reader.records().count()
    };

    Ok(log_index)
}

fn stop_timer(timer: &mut Timer, output_file: &str, index: usize) -> Result<(), std::io::Error> {
    let stopped_time = SystemTime::now();
    let (start_time, paused_duration) =
        read_start_time_and_paused_duration_from_csv(output_file, index)?;

    let elapsed_time = stopped_time.duration_since(start_time).unwrap_or_default();

    timer.update_log_entry_with_elapsed_time(output_file, index, elapsed_time, paused_duration)?;

    println!(
        "Timer stopped at {:?}. Elapsed time: {:?}, Total paused time: {:?}",
        stopped_time,
        elapsed_time.as_secs(),
        paused_duration.as_secs()
    );

    Ok(())
}

fn read_start_time_and_paused_duration_from_csv(
    output_file: &str,
    index: usize,
) -> Result<(SystemTime, Duration), std::io::Error> {
    let file = OpenOptions::new().read(true).open(output_file)?;
    let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));

    for result in reader.records() {
        let record = result?;
        if record.len() >= 5 {
            if let Ok(record_index) = record[0].parse::<usize>() {
                if record_index == index {
                    if let Ok(start_time) = DateTime::parse_from_rfc2822(&record[1]) {
                        let paused_duration = record[4].parse::<u64>().unwrap_or_default();
                        return Ok((
                            start_time.with_timezone(&Utc).into(),
                            Duration::from_secs(paused_duration),
                        ));
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "Invalid start time format in CSV",
                        ));
                    }
                }
            }
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        "No valid start time or paused duration found for the specified index in CSV",
    ))
}

fn get_last_index_from_csv(output_file: &str) -> Result<usize, std::io::Error> {
    let file = OpenOptions::new().read(true).open(output_file)?;
    let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));

    let mut last_index: Option<usize> = None;

    for result in reader.records() {
        let record = result?;
        if let Some(index_str) = record.get(0) {
            if let Ok(index) = index_str.parse::<usize>() {
                last_index = Some(index);
            }
        }
    }

    last_index.ok_or_else(|| Error::new(ErrorKind::Other, "No valid index found in CSV"))
}
