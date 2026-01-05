use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use super::app::{App, Tab};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header + Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(frame.size());

    // Header with tabs
    render_header(frame, app, chunks[0]);

    // Content (active tab)
    match app.active_tab {
        Tab::Dashboard => super::tabs::dashboard::render(frame, &app.dashboard, chunks[1]),
        Tab::Tasks => super::tabs::tasks::render(frame, &mut app.tasks, chunks[1]),
        Tab::Graph => super::tabs::graph::render(frame, &mut app.graph, chunks[1]),
    }

    // Footer (keybindings hint)
    render_footer(frame, chunks[2]); // App not needed
    
    // Help overlay (if active)
    if app.show_help {
        super::widgets::help::render(frame, frame.size());
    }
}

fn render_header(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let titles = vec!["Dashboard", "Tasks", "Graph"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("CueDeck TUI"))
        .select(match app.active_tab {
            Tab::Dashboard => 0,
            Tab::Tasks => 1,
            Tab::Graph => 2,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect) {
    let hint = Span::raw("Tab: Switch | q: Quit | ?: Help");
    let paragraph = Paragraph::new(Line::from(hint));
    frame.render_widget(paragraph, area);
}
