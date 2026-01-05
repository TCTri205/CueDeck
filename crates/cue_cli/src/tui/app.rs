use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use super::tabs::{dashboard::Dashboard, graph::Graph, tasks::Tasks};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Tasks,
    Graph,
}

pub struct App {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub show_help: bool,
    
    // Tab states
    pub dashboard: Dashboard,
    pub tasks: Tasks,
    pub graph: Graph,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            active_tab: Tab::Dashboard,
            should_quit: false,
            show_help: false,
            dashboard: Dashboard::new()?,
            tasks: Tasks::new()?,
            graph: Graph::new()?,
        })
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        // Only process key press events
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.next_tab();
            }
            KeyCode::BackTab => {
                self.previous_tab();
            }
            KeyCode::Char('?') => {
                self.toggle_help();
            }
            _ => {
                // Delegate to active tab
                match self.active_tab {
                    Tab::Dashboard => super::tabs::dashboard::handle_key(key, &mut self.dashboard)?,
                    Tab::Tasks => super::tabs::tasks::handle_key(key, &mut self.tasks)?,
                    Tab::Graph => super::tabs::graph::handle_key(key, &mut self.graph)?,
                }
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Tasks,
            Tab::Tasks => Tab::Graph,
            Tab::Graph => Tab::Dashboard,
        };
    }

    pub fn previous_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Graph,
            Tab::Tasks => Tab::Dashboard,
            Tab::Graph => Tab::Tasks,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_cycling() {
        let mut app = App::new().unwrap();
        assert_eq!(app.active_tab, Tab::Dashboard);

        app.next_tab();
        assert_eq!(app.active_tab, Tab::Tasks);

        app.next_tab();
        assert_eq!(app.active_tab, Tab::Graph);

        app.next_tab();
        assert_eq!(app.active_tab, Tab::Dashboard);
    }

    #[test]
    fn test_previous_tab_cycling() {
        let mut app = App::new().unwrap();
        assert_eq!(app.active_tab, Tab::Dashboard);

        app.previous_tab();
        assert_eq!(app.active_tab, Tab::Graph);

        app.previous_tab();
        assert_eq!(app.active_tab, Tab::Tasks);

        app.previous_tab();
        assert_eq!(app.active_tab, Tab::Dashboard);
    }

    #[test]
    fn test_quit_flag() {
        let mut app = App::new().unwrap();
        assert!(!app.should_quit);

        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_help_toggle() {
        let mut app = App::new().unwrap();
        assert!(!app.show_help);

        app.toggle_help();
        assert!(app.show_help);

        app.toggle_help();
        assert!(!app.show_help);
    }
}
