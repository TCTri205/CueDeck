use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use std::path::PathBuf;
use cue_core::tasks::list_tasks;
use cue_common::Document;

pub struct Tasks {
    state: TableState,
    tasks: Vec<Document>,
    workspace_root: PathBuf,
    show_details: bool,
    detail_content: String,
    detail_title: String,
}

impl Tasks {
    pub fn new() -> Result<Self> {
        let workspace_root = std::env::current_dir()?;
        let mut tasks = Self {
            state: TableState::default(),
            tasks: Vec::new(),
            workspace_root: workspace_root.clone(),
            show_details: false,
            detail_content: String::new(),
            detail_title: String::new(),
        };
        // Initial fetch
        let _ = tasks.refresh(); // Ignore error on initial load to allow UI to start
        Ok(tasks)
    }

    fn refresh(&mut self) -> Result<()> {
        // Fetch all tasks without filtering for now
        let tasks = list_tasks(&self.workspace_root, None, None)?;
        self.tasks = tasks;
        
        // Select first item if list is not empty and nothing selected
        if !self.tasks.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        Ok(())
    }
    
    fn next(&mut self) {
        if self.tasks.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.tasks.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn get_priority_color(priority: &str) -> Color {
        match priority.to_lowercase().as_str() {
            "critical" => Color::Red,
            "high" => Color::LightRed,
            "medium" => Color::Yellow,
            "low" => Color::Green,
            _ => Color::White,
        }
    }

    fn get_status_color(status: &str) -> Color {
        match status.to_lowercase().as_str() {
            "done" => Color::Green,
            "active" => Color::Blue,
            "todo" => Color::Gray,
            "archived" => Color::DarkGray,
            _ => Color::White,
        }
    }

    fn toggle_details(&mut self) -> Result<()> {
        if self.show_details {
            self.show_details = false;
        } else if let Some(i) = self.state.selected() {
            if let Some(doc) = self.tasks.get(i) {
                 // Read file content
                 let content = std::fs::read_to_string(&doc.path).unwrap_or_else(|_| "Error reading file".to_string());
                 self.detail_content = content;
                 self.detail_title = doc.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                 self.show_details = true;
            }
        }
        Ok(())
    }
}

// Standalone render function matching the pattern of other tabs
pub fn render(frame: &mut Frame, tasks: &mut Tasks, area: Rect) {
    let rows: Vec<Row> = tasks.tasks.iter().map(|doc| {
        let meta = doc.frontmatter.as_ref();
        let id = doc.path.file_stem().unwrap_or_default().to_string_lossy();
        let title = meta.map(|m| m.title.clone()).unwrap_or_default();
        let status = meta.map(|m| m.status.clone()).unwrap_or_default();
        let priority = meta.map(|m| m.priority.clone()).unwrap_or_default();
        
        let status_color = Tasks::get_status_color(&status);
        let priority_color = Tasks::get_priority_color(&priority);

        let cells = vec![
            Cell::from(id.into_owned()),
            Cell::from(status).style(Style::default().fg(status_color)),
            Cell::from(priority).style(Style::default().fg(priority_color)),
            Cell::from(title),
        ];
        Row::new(cells)
    }).collect();

    let widths = [
        Constraint::Length(8),  // ID
        Constraint::Length(10), // Status
        Constraint::Length(10), // Priority
        Constraint::Min(20),    // Title
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["ID", "Status", "Priority", "Title"])
            .style(Style::default().add_modifier(Modifier::BOLD).underlined()))
        .block(Block::default().borders(Borders::ALL).title(" Tasks "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");
        
    frame.render_stateful_widget(table, area, &mut tasks.state);

    if tasks.show_details {
        let block = Block::default()
            .title(format!(" Task: {} ", tasks.detail_title))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        
        let area = centered_rect(area, 80, 80);
        let paragraph = ratatui::widgets::Paragraph::new(tasks.detail_content.clone())
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false }); // Wrap text
            
        // Clear area first to avoid transparency
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(paragraph, area);
    }
}

// Helper for centering popup
fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Standalone handle_key function matching the pattern of other tabs
pub fn handle_key(key: KeyEvent, tasks: &mut Tasks) -> Result<()> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !tasks.show_details { tasks.next(); } // Lock nav if popup open
        },
        KeyCode::Char('k') | KeyCode::Up => {
             if !tasks.show_details { tasks.previous(); }
        },
        KeyCode::Char('r') => tasks.refresh()?,
        KeyCode::Enter => tasks.toggle_details()?,
        KeyCode::Esc => {
            if tasks.show_details {
                tasks.toggle_details()?;
            }
        },
        _ => {}
    }
    Ok(())
}
