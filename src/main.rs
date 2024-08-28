use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    // start new activity
    #[arg(short = 's')]
    start: bool,

    #[arg(short = 'd')]
    description: bool,
    /// project to which the new activity belongs    
    #[arg(short = 'p')]
    project: bool,
    #[arg(name = "FILE")]
    /// file in which util tracks all the activities    
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{args:?}");

     let exit_code = 0;

    std::process::exit(exit_code)
}
