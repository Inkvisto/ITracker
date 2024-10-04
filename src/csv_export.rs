use chrono::{DateTime, Utc};
use std::{
    fs::OpenOptions,
    io::{Error as IoError, Write},
    time::Duration,
};

/// Structure representing a task log entry.
pub struct TaskLog {
    /// The description or title of the task.
    data: String,
    /// The start time of the task log entry.
    start_time: DateTime<Utc>,
    /// The duration of the task.
    duration: Duration,
}

impl TaskLog {
    /// Creates a new instance of `TaskLog`.
    ///
    /// # Arguments
    /// - `data`: A string containing the task description or title.
    /// - `start_time`: The start time of the task in UTC.
    /// - `duration`: The duration of the task.
    ///
    /// # Returns
    /// - `Self`: A new `TaskLog` instance.
    pub fn new(data: String, start_time: DateTime<Utc>, duration: Duration) -> Self {
        TaskLog {
            data,
            start_time,
            duration,
        }
    }

    /// Exports the task logs to a CSV file.
    ///
    /// This function opens a CSV file for writing (creating it if necessary)
    /// and appends the task logs as rows. The CSV file includes a header
    /// row for the data fields.
    ///
    /// # Arguments
    /// - `logs`: A slice of `TaskLog` entries to be exported.
    /// - `output_file`: The path to the output CSV file.
    ///
    /// # Returns
    /// - `Result<(), IoError>`: An empty result on success or an error if the file operation fails.
    pub fn export_to_csv(logs: &[TaskLog], output_file: &str) -> Result<(), IoError> {
        let mut file = OpenOptions::new()
            .create(true) // Create the file if it doesn't exist
            .append(true) // Append to the file
            .open(output_file)?;

        // Write the header if the file is empty
        writeln!(file, "Title,Description,Start Time,Duration (secs)")?;

        // Write each task log as a CSV row
        for log in logs {
            writeln!(
                file,
                "{},{},{}",
                log.data,
                log.start_time.to_rfc2822(),
                log.duration.as_secs()
            )?;
        }

        Ok(())
    }
}
