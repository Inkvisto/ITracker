use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Timezone for logging
    #[arg(short = 'z', long = "timezone", default_value = "UTC")]
    pub timezone: String,

    // Path to the log file
    #[clap(short, long)]
    pub log: Option<String>,

    /// Output file for tracking activities
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<PathBuf>,

    /// Start the timer
    #[arg(short = 's', action = clap::ArgAction::SetTrue)]
    pub start: bool,

    /// Stop the timer
    #[arg(short = 't', action = clap::ArgAction::SetTrue)]
    pub stop: bool,

    /// Add a new task
    #[arg(short = 'a', long="add",action = clap::ArgAction::SetTrue)]
    pub add: bool,

    /// Pause the timer
    #[arg(short = 'p', long = "pause", action = clap::ArgAction::SetTrue)]
    pub pause: bool,

    /// Resume the timer
    #[arg(short = 'r', long = "resume", action = clap::ArgAction::SetTrue)]
    pub resume: bool,

    /// Show the daily summary
    #[arg(long = "show-daily-summary", action = clap::ArgAction::SetTrue)]
    pub show_daily_summary: bool,

    /// Show the weekly summary
    #[arg(long = "show-weekly-summary", action = clap::ArgAction::SetTrue)]
    pub show_weekly_summary: bool,

    /// Delete a specific log entry by index
    #[arg(short = 'd', long = "delete-log", value_name = "INDEX")]
    pub delete_log: Option<usize>,

    /// Export logs to CSV
    #[arg(long = "export-to-csv", action = clap::ArgAction::SetTrue)]
    pub export_to_csv: bool,
}
