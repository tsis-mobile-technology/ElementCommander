mod ai;
mod app;
mod commands;
mod config;
mod events;
mod fs;
mod ops;
mod panel;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hermes_tail.log")
        .unwrap_or_else(|_| std::fs::File::create("/tmp/hermes_tail.log").unwrap());

    tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("=== hermes_tail 시작 ===");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Create and run app
    let app = app::App::new()?;
    let result = app.run(terminal).await;

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    tracing::info!("=== hermes_tail 종료 ===");

    result
}
