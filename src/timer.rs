use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, WriterBuilder};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Error, ErrorKind},
    time::{Duration, SystemTime},
};

pub trait TaskLog {
    fn log_task(&mut self, data: &str, output_file: &str) -> Result<(), std::io::Error>;
}

pub struct Timer {
    pub pause_duration: Duration,
    pub is_paused: bool,
    paused_time: Option<SystemTime>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            pause_duration: Duration::new(0, 0),
            is_paused: false,
            paused_time: None,
        }
    }

    pub fn pause(&mut self, output_file: &str, index: usize) -> Result<(), std::io::Error> {
        if !self.is_paused {
            self.paused_time = Some(SystemTime::now());
            self.is_paused = true;

            // Read the start time from the file for the specified index
            let start_time = self.read_start_time_from_csv(output_file, index)?;

            // Write the paused time to the file
            if let Some(paused_time) = self.paused_time {
                let paused_duration = paused_time.duration_since(start_time).unwrap_or_default();
                // Update the log entry in the CSV file with the paused duration
                self.update_log_entry_with_paused_time(output_file, index, paused_duration)?;
            }
        }
        Ok(())
    }

    pub fn resume(&mut self, output_file: &str, index: usize) -> Result<(), std::io::Error> {
        if self.is_paused {
            // Read the start time from the file for the specified index
            let start_time = self.read_start_time_from_csv(output_file, index)?;

            // Calculate paused duration
            if let Some(paused_time) = self.paused_time {
                let pause_duration = paused_time.duration_since(start_time).unwrap_or_default();
                self.pause_duration += pause_duration; // Update total paused duration
            }

            self.is_paused = false; // Reset paused state
            self.paused_time = None; // Reset paused time
        }
        Ok(())
    }

    pub fn get_elapsed_time(
        &self,
        output_file: &str,
        index: usize,
    ) -> Result<Duration, std::io::Error> {
        // Read the start time from the file for the specified index
        let start_time = self.read_start_time_from_csv(output_file, index)?;

        if self.is_paused {
            return Ok(self.pause_duration);
        }

        // Calculate the elapsed time
        let elapsed = start_time.elapsed().unwrap_or_default() - self.pause_duration;
        Ok(elapsed)
    }

    /// Reads the start time from the CSV file for the given index.
    fn read_start_time_from_csv(
        &self,
        output_file: &str,
        index: usize,
    ) -> Result<SystemTime, std::io::Error> {
        let file = OpenOptions::new().read(true).open(output_file)?;
        let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));

        for result in reader.records() {
            let record = result?;
            if record.len() >= 5 {
                // Parse the index from the first field
                if let Ok(record_index) = record[0].parse::<usize>() {
                    if record_index == index {
                        // Parse the start time from the second field
                        if let Ok(start_time) = DateTime::parse_from_rfc2822(&record[1]) {
                            return Ok(start_time.with_timezone(&Utc).into());
                        }
                    }
                }
            }
        }
        Err(Error::new(
            ErrorKind::NotFound,
            "Start time not found for the specified index",
        ))
    }

    pub fn update_log_entry_with_elapsed_time(
        &self,
        output_file: &str,
        index: usize,
        elapsed_time: Duration,
        paused_time: Duration,
    ) -> Result<(), std::io::Error> {
        let mut records = self.read_csv_records(output_file)?;

        // Modify the specific log entry with the elapsed time and paused duration
        if let Some(record) = records.get_mut(index.saturating_sub(1)) {
            // index - 1 to adjust for zero-based index
            if record.len() >= 5 {
                // Update Elapsed Time
                record[3] = elapsed_time.as_secs().to_string();
                // Update the paused duration
                record[4] = paused_time.as_secs().to_string();
            } else {
                // If there are not enough fields, create a valid record
                record.push(paused_time.as_secs().to_string());
            }
        }

        self.write_csv_records(output_file, &records)?;

        Ok(())
    }

    pub fn update_log_entry_with_paused_time(
        &self,
        output_file: &str,
        index: usize,
        paused_duration: Duration,
    ) -> Result<(), std::io::Error> {
        let mut records = self.read_csv_records(output_file)?;

        // Modify the specific log entry with the paused duration
        if let Some(record) = records.get_mut(index.saturating_sub(1)) {
            if record.len() >= 5 {
                // Update the paused duration in the CSV
                record[4] = paused_duration.as_secs().to_string();
            }
        }

        self.write_csv_records(output_file, &records)?;

        Ok(())
    }

    fn read_csv_records(&self, output_file: &str) -> Result<Vec<Vec<String>>, std::io::Error> {
        let file = OpenOptions::new().read(true).open(output_file)?;
        let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));
        let mut records = Vec::new();

        // Read the CSV records
        for result in reader.records() {
            let record = result?;
            records.push(record.iter().map(|s| s.to_string()).collect());
        }

        Ok(records)
    }

    fn write_csv_records(
        &self,
        output_file: &str,
        records: &[Vec<String>],
    ) -> Result<(), std::io::Error> {
        // Write the updated records back to the CSV file
        let file = OpenOptions::new()
            .write(true)
            .truncate(true) // Clear the file before writing
            .open(output_file)?;

        let mut writer = WriterBuilder::new().from_writer(BufWriter::new(file));

        // Write headers (including paused duration)
        writer.write_record(&[
            "Index",
            "Start Time",
            "Task Description",
            "Elapsed Time (seconds)",
            "Paused Duration (seconds)",
        ])?;

        // Write the updated records
        for record in records {
            writer.write_record(record)?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl TaskLog for Timer {
    fn log_task(&mut self, data: &str, output_file: &str) -> Result<(), std::io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(output_file)?;

        let is_empty = file.metadata()?.len() == 0;

        let mut writer = csv::Writer::from_writer(BufWriter::new(file));

        if is_empty {
            writer.write_record(&[
                "Index",
                "Start Time",
                "Task Description",
                "Elapsed Time (seconds)",
                "Paused Duration (seconds)",
            ])?;
        }

        let current_index = {
            let mut reader = csv::Reader::from_reader(BufReader::new(File::open(output_file)?));
            reader.records().count() // Count the total number of records
        };

        let index = current_index + 1;

        writer.write_record(&[
            index.to_string(),
            Utc::now().to_rfc2822(),
            data.to_string(),
            "0".to_string(), // Elapsed time, initialized to 0
            "0".to_string(), // Paused duration, initialized to 0
        ])?;

        writer.flush()?;
        Ok(())
    }
}
