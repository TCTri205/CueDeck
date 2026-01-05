use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use super::app::App;

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle events, returns true if should quit
    pub fn handle_events(&mut self, app: &mut App) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Global quit on Ctrl+C
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.quit();
                        return Ok(true);
                    }
                    
                    // Delegate rest to app
                    app.on_key(key)?;
                }
        }
        Ok(app.should_quit)
    }
}
