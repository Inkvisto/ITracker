# Rust Tracking Program

## Overview

This Rust-based logging program is designed to help users track tasks and their associated elapsed time, providing a structured way to log activities and monitor productivity. 
Each log entry includes a timestamp, a description of the task, and the duration it took to complete.

## Features

- **Log Task**: Easily log tasks with their start time and duration.
- **Indexing**: Each log entry is assigned a unique index, allowing for easy reference and management of entries.
- **Flexible Logging**: Append new log entries to an existing log file, ensuring persistence between program runs.
- **Deletion of Log Entries**: Remove entire log entries along with their details, based on the index.
- **Terminal Width Handling**: Dynamically adjusts output to fit the terminal width.
- **Summary View**: Retrieve summaries of logged tasks (not detailed here, but can be added).

## Technologies Used
1. **Rust**: The primary programming language used for the project.
2. **Chrono**: A date and time library for Rust, used for managing timestamps and durations.
3. **Clap**: A command-line argument parser for Rust, allowing for easy handling of command-line options and flags.
4. **Config**: A configuration library for Rust, enabling the management of configuration files.
5. **Crossterm**: A cross-platform library for terminal manipulation, used for handling user input and terminal output.
6. **Ratatui**: A library for building rich text user interfaces in Rust, providing tools for layout and rendering.
7. **Serde**: A framework for serializing and deserializing Rust data structures, enabling the conversion between Rust types and formats like JSON and TOML.
8. **TOML**: A parser for TOML files, used for reading configuration files.
9. **Tui-textarea**: A library for creating text areas in terminal user interfaces, facilitating user input.
