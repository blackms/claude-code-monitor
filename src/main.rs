mod app;
mod config;
mod data;
mod event;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use config::Config;
use event::{handle_key_event, AppEvent, EventHandler};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let config = Config::new()?;
    let watch_paths = vec![config.stats_file.clone(), config.history_file.clone()];
    let tick_rate = config.refresh_rate;
    let quota_refresh_rate = Duration::from_secs(2); // Refresh quota every 2 seconds

    let mut app = App::new(config);
    app.load_data();
    app.load_quota(); // Load quota on startup

    // Event handler
    let events = EventHandler::new(tick_rate, watch_paths, quota_refresh_rate)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app, &events);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    events: &EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        match events.next()? {
            AppEvent::Key(key) => handle_key_event(key, app),
            AppEvent::FileChanged(_) => {
                if app.is_live {
                    app.load_data();
                }
            }
            AppEvent::QuotaRefresh => {
                if app.is_live {
                    app.load_quota();
                }
            }
            AppEvent::Tick => {
                // UI refresh happens on each tick
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
