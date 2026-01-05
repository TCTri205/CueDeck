use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect) {
    // Center the help overlay
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1]);

    let popup_area = horizontal_layout[1];

    // Help content
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            "  ".into(),
            "Tab".into(),
            " / ".into(),
            "Shift+Tab".into(),
            "  -  Switch between tabs".into(),
        ]),
        Line::from(""),
        Line::from(vec!["  ".into(), "j".into(), " / ".into(), "↓".into(), "       -  Move down (in lists)".into()]),
        Line::from(vec!["  ".into(), "k".into(), " / ".into(), "↑".into(), "       -  Move up (in lists)".into()]),
        Line::from(""),
       Line::from(vec!["  ".into(), "Enter".into(), "       -  Select item / View details".into()]),
        Line::from(""),
        Line::from(vec!["  ".into(), "?".into(), "           -  Toggle this help".into()]),
        Line::from(vec!["  ".into(), "q".into(), " / ".into(), "Ctrl+C".into(), "   -  Quit".into()]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Keybindings")
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));

    // Clear background and render popup
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}
