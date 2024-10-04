use crate::csv_export::TaskLog;
use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Error as IoError, Write},
    time::{Duration, SystemTime},
};

/// A struct to represent a Timer that tracks elapsed time, pause duration,
/// daily logs, and task logs.
pub struct Timer {
    start_time: Option<SystemTime>,
    stop_time: Option<SystemTime>,
    pause_duration: Duration,
    daily_log: HashMap<NaiveDate, Duration>,
    task_logs: Vec<TaskLog>,
}

impl Timer {
    /// Creates a new Timer instance.
    pub fn new() -> Self {
        Self {
            start_time: None,
            stop_time: None,
            pause_duration: Duration::default(),
            daily_log: HashMap::new(),
            task_logs: Vec::new(),
        }
    }

    /// Starts the timer by recording the current time.
    pub fn start(&mut self) {
        self.start_time = Some(SystemTime::now());
        self.pause_duration = Duration::default(); // Reset pause duration
    }

    /// Stops the timer and records the stop time.
    pub fn stop(&mut self) {
        self.stop_time = Some(SystemTime::now());
    }

    /// Returns the start time of the timer if it has been started.
    ///
    /// # Returns
    /// An `Option<SystemTime>` containing the start time or `None` if not started.
    pub fn started_time(&self) -> Option<SystemTime> {
        self.start_time
    }

    /// Returns the stop time of the timer if it has been stopped.
    ///
    /// # Returns
    /// An `Option<SystemTime>` containing the stop time or `None` if not stopped.
    pub fn stopped_time(&self) -> Option<SystemTime> {
        self.stop_time
    }

    /// Pauses the timer and records the duration since it started.
    pub fn pause(&mut self) {
        if let Some(start) = self.start_time {
            self.pause_duration += SystemTime::now().duration_since(start).unwrap_or_default();
            self.start_time = None;
        }
    }

    /// Resumes the timer from a paused state.
    pub fn resume(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(SystemTime::now());
        }
    }

    /// Returns the total elapsed time including any pause duration.
    ///
    /// # Returns
    /// A `Duration` representing the total elapsed time.
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            SystemTime::now().duration_since(start).unwrap_or_default() + self.pause_duration
        } else {
            self.pause_duration
        }
    }

    /// Logs the daily time spent on tasks.
    ///
    /// This method calculates the time spent from the start time to the stop time
    /// (or now if not stopped), subtracting any pause duration, and logs it
    /// under the current date.
    pub fn log_daily_time(&mut self) -> Result<(), IoError> {
        if let Some(start) = self.start_time {
            let stop = self.stop_time.unwrap_or(SystemTime::now());
            let elapsed = stop.duration_since(start).unwrap_or_default() - self.pause_duration;
            let current_date = Local::now().date_naive();
            *self
                .daily_log
                .entry(current_date)
                .or_insert(Duration::default()) += elapsed;
        }
        Ok(())
    }

    /// Saves the daily log to a specified output file.
    ///
    /// # Arguments
    /// * `output_file` - A string slice that holds the path to the output file.
    ///
    /// # Returns
    /// * `Result<(), IoError>` - An empty result on success or an error.
    pub fn save_daily_log(&self, output_file: &str) -> Result<(), IoError> {
        let mut file = self.open_file(output_file)?;
        for (date, duration) in &self.daily_log {
            writeln!(file, "{},{}", date, duration.as_secs())?;
        }
        Ok(())
    }

    /// Returns a clone of the daily log summary.
    ///
    /// # Returns
    /// A `HashMap<NaiveDate, Duration>` representing the daily log summary.
    pub fn get_daily_summary(&self) -> HashMap<NaiveDate, Duration> {
        self.daily_log.clone()
    }

    /// Returns a summary of the logged time for the current week.
    ///
    /// # Returns
    /// A `HashMap<NaiveDate, Duration>` representing the weekly log summary.
    pub fn get_weekly_summary(&self) -> HashMap<NaiveDate, Duration> {
        let today = Local::now().date_naive();
        let start_of_week =
            today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

        self.daily_log
            .iter()
            .filter(|(&date, _)| date >= start_of_week)
            .map(|(date, duration)| (*date, *duration))
            .collect()
    }

    /// Exports task logs to a CSV file.
    ///
    /// # Arguments
    /// * `output_file` - A string slice that holds the path to the output file.
    ///
    /// # Returns
    /// * `Result<(), IoError>` - An empty result on success or an error.
    pub fn export_logs_to_csv(&self, output_file: &str) -> Result<(), IoError> {
        TaskLog::export_to_csv(&self.task_logs, output_file)
    }

    /// Logs a task with the specified data and duration to the output file.
    ///
    /// # Arguments
    /// * `data` - A string slice containing task details.
    /// * `output_file` - A string slice that holds the path to the output file.
    /// * `program_start_time` - The time when the program started.
    ///
    /// # Returns
    /// * `Result<(), IoError>` - An empty result on success or an error.
    pub fn log_task(
        &mut self,
        data: &str,
        output_file: &str,
        program_start_time: SystemTime,
    ) -> Result<(), IoError> {
        let datetime: DateTime<Utc> = program_start_time.into();
        let duration = self.elapsed();

        let task_log = TaskLog::new(data.to_string(), datetime, duration);
        self.task_logs.push(task_log);

        // Read the file to find the last used index
        let mut current_index = 0;
        if let Ok(file) = File::open(output_file) {
            let reader = BufReader::new(file);

            // Search for the highest existing log entry index
            for line in reader.lines() {
                let line = line?;
                if line.starts_with(" Log Entry: ") {
                    if let Some(entry) = line.split_whitespace().last() {
                        if let Ok(parsed_index) = entry.parse::<usize>() {
                            if parsed_index > current_index {
                                current_index = parsed_index;
                            }
                        }
                    }
                }
            }
        }

        // Increment the index for the new log entry
        let index = current_index + 1;

        let mut file = self.open_file(output_file)?;
        let width = self.get_terminal_width();
        let line = "-".repeat(width);

        // Write the new log entry with the next index
        writeln!(file, "\n Log Entry: {}", index)?; // Add incremented index here
        writeln!(file, " Start Time: {}", datetime.to_rfc2822())?;
        writeln!(file, " {}", data)?;
        writeln!(file, " Elapsed Time: {} seconds", duration.as_secs())?;
        writeln!(file, "{}", line)?;

        Ok(())
    }

    /// Opens a file for appending log entries.
    ///
    /// # Arguments
    /// * `output_file` - A string slice that holds the path to the output file.
    ///
    /// # Returns
    /// * `Result<File, IoError>` - A result containing the opened file or an error.
    fn open_file(&self, output_file: &str) -> Result<File, IoError> {
        Ok(OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_file)?)
    }

    /// Retrieves the terminal width, defaulting to 80 if unavailable.
    ///
    /// # Returns
    /// A `usize` representing the width of the terminal.
    fn get_terminal_width(&self) -> usize {
        std::env::var("COLUMNS")
            .unwrap_or_else(|_| "80".to_string())
            .parse::<usize>()
            .unwrap_or(80)
    }
}

/// Function to log elapsed time to a file.
///
/// # Arguments
/// * `elapsed` - A `Duration` representing the elapsed time to log.
/// * `output_file` - A string slice that holds the path to the output file.
///
/// # Returns
/// * `Result<(), IoError>` - An empty result on success or an error.
pub fn log_elapsed_time(elapsed: Duration, output_file: &str) -> Result<(), IoError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_file)?;

    writeln!(file, "Elapsed time: {:?} seconds", elapsed.as_secs())?;

    Ok(())
}
