use csv::{ReaderBuilder, WriterBuilder};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter};

/// Represents a single log entry with an index, start time, message, elapsed time, and paused time.
#[derive(Debug)]
pub struct LogEntry {
    pub index: usize,         // Index of the log entry
    pub start_time: String,   // Start time of the log entry
    pub message: String,      // Message associated with the log entry
    pub elapsed_time: String, // Elapsed time recorded in the log entry
    pub paused_time: String,  // Paused time recorded in the log entry
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
    // Open the CSV file for reading
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));

    let mut entries = Vec::new();

    // Iterate over each record in the CSV file
    for result in reader.records() {
        let record = result.map_err(|e| io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Parse each field from the CSV into the LogEntry struct
        let entry = LogEntry {
            index: record[0]
                .parse::<usize>()
                .map_err(|e| io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            start_time: record[1].to_string(),
            message: record[2].to_string(),
            elapsed_time: record[3].to_string(),
            paused_time: record.get(4).unwrap_or(&"0".to_string()).to_string(), // Default to "0" if not present
        };
        entries.push(entry);
    }

    Ok(entries)
}

/// Deletes a log entry by its index from the specified log file.
///
/// This function searches for a log entry by its index and removes it along with
/// the associated information (up to the next delimiter).
///
/// # Arguments
/// - `log_file`: The path to the log file.
/// - `index`: The index of the log entry to delete.
///
/// # Returns
/// - `Ok(())`: If the deletion is successful.
/// - `Err(std::io::Error)`: An error if file operations fail.
pub fn delete_log_entry(log_file: &str, index: usize) -> Result<(), io::Error> {
    // Open the CSV file for reading
    let file = File::open(log_file)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));

    // Read all existing records and filter out the entry with the specified index
    let mut updated_records = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let record_index: usize = record[0]
            .parse()
            .map_err(|e| io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if record_index != index {
            updated_records.push(record.clone());
        }
    }

    // Open the CSV file for writing (truncate it to start fresh)
    let output_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(log_file)?;

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(BufWriter::new(output_file));

    // Write the header to the CSV file
    writer.write_record(&[
        "Index",
        "Start Time",
        "Task Description",
        "Elapsed Time (seconds)",
        "Paused Time (seconds)",
    ])?;

    // Write the remaining records back to the file
    for record in updated_records {
        writer
            .write_record(&record)
            .map_err(|e| io::Error::new(std::io::ErrorKind::WriteZero, e))?;
    }

    writer.flush()?;
    Ok(())
}
