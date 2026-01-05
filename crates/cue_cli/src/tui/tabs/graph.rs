use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::path::PathBuf;
use cue_core::task_graph::TaskGraph;
use cue_core::tasks::list_tasks;

pub struct Graph {
    task_graph: TaskGraph,
    state: ListState,
    tasks_list: Vec<String>,
    titles: HashMap<String, String>,
    workspace_root: PathBuf,
}

impl Graph {
    pub fn new() -> Result<Self> {
        let workspace_root = std::env::current_dir()?;
        let mut graph = Self {
            task_graph: TaskGraph::new(),
            state: ListState::default(),
            tasks_list: Vec::new(),
            titles: HashMap::new(),
            workspace_root,
        };
        let _ = graph.refresh();
        Ok(graph)
    }

    fn refresh(&mut self) -> Result<()> {
        // Build graph
        self.task_graph = TaskGraph::from_workspace(&self.workspace_root)?;

        // Fetch tasks to get titles
        let tasks = list_tasks(&self.workspace_root, None, None)?;
        
        self.titles.clear();
        self.tasks_list.clear();

        for doc in tasks {
            let id = doc.path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let title = doc.frontmatter.map(|m| m.title).unwrap_or_else(|| id.clone());
            self.titles.insert(id.clone(), title);
            self.tasks_list.push(id);
        }

        // Sort tasks for list view
        self.tasks_list.sort();

        // Ensure selection is valid
        if !self.tasks_list.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }

        Ok(())
    }

    fn next(&mut self) {
        if self.tasks_list.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks_list.len() - 1 {
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
        if self.tasks_list.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn render(frame: &mut Frame, graph: &mut Graph, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // List of tasks
            Constraint::Percentage(70), // Details
        ])
        .split(area);

    // Left Panel: List of Tasks
    let items: Vec<ListItem> = graph.tasks_list
        .iter()
        .map(|id| {
            let title = graph.titles.get(id).map(|s| s.as_str()).unwrap_or(id);
            ListItem::new(format!("{} - {}", id, title))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Tasks "))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, chunks[0], &mut graph.state);

    // Right Panel: Details
    let selected_id = graph.state.selected().and_then(|i| graph.tasks_list.get(i));

    if let Some(id) = selected_id {
        let title = graph.titles.get(id).map(|s| s.as_str()).unwrap_or(id);
        
        // Get dependencies
        let deps = graph.task_graph.get_dependencies(id);
        let dependents = graph.task_graph.get_dependents(id);

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Task: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{} ({})", title, id), Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Depends On:", Style::default().fg(Color::Yellow))),
        ];

        if deps.is_empty() {
             lines.push(Line::from("  (None)"));
        } else {
            for dep_id in deps {
                let dep_title = graph.titles.get(&dep_id).map(|s| s.as_str()).unwrap_or(&dep_id);
                lines.push(Line::from(format!("  -> {} ({})", dep_title, dep_id)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Needed By:", Style::default().fg(Color::Blue))));

        if dependents.is_empty() {
            lines.push(Line::from("  (None)"));
        } else {
            for dep_id in dependents {
                let dep_title = graph.titles.get(&dep_id).map(|s| s.as_str()).unwrap_or(&dep_id);
                lines.push(Line::from(format!("  <- {} ({})", dep_title, dep_id)));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Relations "));
        
        frame.render_widget(paragraph, chunks[1]);

    } else {
        let paragraph = Paragraph::new("Select a task to view relationships")
            .block(Block::default().borders(Borders::ALL).title(" Relations "));
        frame.render_widget(paragraph, chunks[1]);
    }
}

pub fn handle_key(key: KeyEvent, graph: &mut Graph) -> Result<()> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => graph.next(),
        KeyCode::Char('k') | KeyCode::Up => graph.previous(),
        KeyCode::Char('r') => graph.refresh()?,
        _ => {}
    }
    Ok(())
}
