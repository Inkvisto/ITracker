use crate::log::LogEntry;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tui_textarea::{Input, Key, TextArea};

/// Renders the logs in a terminal interface.
///
/// # Arguments
/// * `logs` - An optional vector of `LogEntry` items to display in the terminal.
///
/// # Returns
/// * `io::Result<Vec<String>>` - A result containing a vector of strings entered in the textarea, or an error.
pub fn render(logs: Option<Vec<LogEntry>>) -> io::Result<Vec<String>> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    // Enable raw mode and set up the terminal
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Write your task"),
    );

    if let Some(logs) = logs {
        let mut start_index = 0;

        // Main loop for handling input and rendering
        loop {
            terminal.draw(|f| {
                let size = f.area();
                let visible_count = (size.height / 6).min(logs.len() as u16); // Adjust this number based on your terminal size
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        (0..visible_count)
                            .map(|_| Constraint::Min(1))
                            .collect::<Vec<_>>(),
                    );

                let chunks = layout.split(size);

                // Render only the visible log entries
                for (i, log) in logs
                    .iter()
                    .enumerate()
                    .skip(start_index)
                    .take(visible_count.into())
                {
                    let log_block = Block::default()
                        .title(format!("Log Entry {}", log.index))
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black).fg(Color::White));

                    // Format log details with newlines
                    let log_details = format!(
                        "Start Time: {}\nMessage:\n{}\nElapsed Time: {}",
                        log.start_time.trim(),
                        log.message.trim(),
                        log.elapsed_time.trim()
                    );

                    let log_paragraph = Paragraph::new(log_details).block(log_block);
                    f.render_widget(log_paragraph, chunks[i - start_index as usize]);
                    // Adjust the index for visible entries
                }
            })?;

            // Handle input for exiting the loop
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break, // Exit on Esc key
                    KeyCode::Down => {
                        // Scroll down
                        if start_index + 1 < logs.len() {
                            start_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        // Scroll up
                        if start_index > 0 {
                            start_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        // If no logs are provided, enter input mode
        loop {
            terminal.draw(|f| {
                f.render_widget(&textarea, f.area());
            })?;
            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                input => {
                    textarea.input(input);
                }
            }
        }
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print the lines from the textarea and return them
    let lines: Vec<String> = textarea.lines().iter().cloned().collect();
    Ok(lines)
}
