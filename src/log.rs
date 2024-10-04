use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Represents a single log entry with an index, start time, message, and elapsed time.
#[derive(Debug)]
pub struct LogEntry {
    pub index: usize,         // Index of the log entry
    pub start_time: String,   // Start time of the log entry
    pub message: String,      // Message associated with the log entry
    pub elapsed_time: String, // Elapsed time recorded in the log entry
}

/// Reads logs from a specified file and returns a vector of `LogEntry`.
///
/// # Arguments
/// - `file_path`: The path to the log file.
///
/// # Returns
/// - `Ok(Vec<LogEntry>)`: A vector of log entries if successful.
/// - `Err(std::io::Error)`: An error if file operations fail.
pub fn read_logs_from_file(file_path: &str) -> Result<Vec<LogEntry>, std::io::Error> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_entry: Option<LogEntry> = None;

    for line in reader.lines() {
        let line = line?;

        if line.trim().starts_with("Log Entry:") {
            // If we have a current entry, save it before creating a new one
            if let Some(entry) = current_entry.take() {
                entries.push(entry);
            }

            // Create a new log entry and extract the index
            let index = line.trim()[10..].trim().parse::<usize>().unwrap_or(0);
            current_entry = Some(LogEntry {
                index,
                start_time: String::new(),
                message: String::new(),
                elapsed_time: String::new(),
            });
        } else if line.trim().starts_with("Start Time:") {
            if let Some(ref mut entry) = current_entry {
                entry.start_time = line.trim()[12..].trim().to_string();
            }
        } else if line.trim().starts_with("Elapsed Time:") {
            if let Some(ref mut entry) = current_entry {
                entry.elapsed_time = line.trim()[14..].trim().to_string();
            }
        } else if !line.trim().starts_with("----") {
            // Ignore lines that are just delimiters
            // This line is considered part of the message
            if let Some(ref mut entry) = current_entry {
                entry.message.push_str(&format!("{}\n", line.trim()));
            }
        }
    }

    // Push the last entry if it exists and is not empty
    if let Some(entry) = current_entry {
        if !entry.start_time.is_empty()
            || !entry.message.is_empty()
            || !entry.elapsed_time.is_empty()
        {
            entries.push(entry);
        }
    }
    Ok(entries)
}

/// Deletes a log entry by its index from the specified log file.
///
/// This function searches for a log entry by its index, and removes it along with
/// the associated information (up to the next delimiter).
///
/// # Arguments
/// - `log_file`: The path to the log file.
/// - `index`: The index of the log entry to delete.
///
/// # Returns
/// - `Ok(())`: If the deletion is successful.
/// - `Err(std::io::Error)`: An error if file operations fail.
pub fn delete_log_entry(log_file: &str, index: usize) -> Result<(), std::io::Error> {
    let file = File::open(log_file)?;
    let reader = BufReader::new(file);

    // Collect all the lines from the log file
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Prepare to store the updated logs
    let mut updated_logs = Vec::new();
    let mut delete_mode = false;

    for line in lines.iter() {
        if delete_mode {
            // If we are in delete mode, skip lines until we hit the delimiter
            if line.trim().starts_with("----") {
                delete_mode = false; // End delete mode after processing the delimiter
            }
            continue; // Skip all lines in delete mode
        }

        // Look for the log entry to delete
        if line.trim() == format!("Log Entry: {}", index) {
            delete_mode = true; // Start deleting lines once the log entry is found
            continue; // Skip the line with "Log Entry: <index>"
        }

        // Add the line to the updated logs if it's not part of the deleted block
        updated_logs.push(line.clone());
    }

    // Overwrite the log file with the remaining entries
    let output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(log_file)?;

    let mut writer = BufWriter::new(output_file);

    // Write the remaining lines back to the file
    for line in updated_logs {
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}
