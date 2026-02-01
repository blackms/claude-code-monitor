use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    FileChanged(#[allow(dead_code)] PathBuf),
    Tick,
    QuotaRefresh,
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    _watcher: Option<RecommendedWatcher>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, watch_paths: Vec<PathBuf>, quota_refresh_rate: Duration) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        // File watcher
        let tx_watcher = tx.clone();
        let watcher = if !watch_paths.is_empty() {
            let mut watcher = RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        for path in event.paths {
                            let _ = tx_watcher.send(AppEvent::FileChanged(path));
                        }
                    }
                },
                NotifyConfig::default().with_poll_interval(Duration::from_millis(500)),
            )?;

            for path in &watch_paths {
                if path.exists() {
                    watcher.watch(path, RecursiveMode::NonRecursive)?;
                }
            }

            Some(watcher)
        } else {
            None
        };

        // Keyboard + tick handler thread
        let tx_keys = tx.clone();
        std::thread::spawn(move || {
            let mut last_quota_refresh = Instant::now();

            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        if tx_keys.send(AppEvent::Key(key)).is_err() {
                            break;
                        }
                    }
                } else {
                    // Check if it's time to refresh quota
                    if last_quota_refresh.elapsed() >= quota_refresh_rate {
                        if tx_keys.send(AppEvent::QuotaRefresh).is_err() {
                            break;
                        }
                        last_quota_refresh = Instant::now();
                    }

                    if tx_keys.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Self { rx, _watcher: watcher })
    }

    pub fn next(&self) -> anyhow::Result<AppEvent> {
        Ok(self.rx.recv()?)
    }
}

pub fn handle_key_event(key: KeyEvent, app: &mut crate::app::App) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('l') => app.toggle_live(),
        KeyCode::Tab => app.next_panel(),
        KeyCode::BackTab => app.prev_panel(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
        _ => {}
    }
}
