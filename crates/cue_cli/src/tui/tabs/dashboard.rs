use cue_core::doctor::{gather_workspace_stats, WorkspaceStats};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::env;

pub struct Dashboard {
    recent_files: Vec<String>,
    stats: Option<WorkspaceStats>,
    _error: Option<String>,
}

impl Dashboard {
    pub fn new() -> anyhow::Result<Self> {
        let cwd = env::current_dir()?;
        
        // Fetch stats
        let stats = gather_workspace_stats(&cwd).ok();
        
        // Fetch recent files (simple implementation: list .md files in cards/ sorted by mtime)
        let recent_files = fetch_recent_files(&cwd).unwrap_or_default();

        Ok(Self {
            recent_files,
            stats,
            _error: None,
        })
    }
}

fn fetch_recent_files(root: &std::path::Path) -> anyhow::Result<Vec<String>> {
    use walkdir::WalkDir;
    let mut files: Vec<(String, std::time::SystemTime)> = Vec::new();

    let cuedeck_root = root.join(".cuedeck");
    if cuedeck_root.exists() {
        for entry in WalkDir::new(cuedeck_root).max_depth(3) {
            let entry = entry?;
            if entry.file_type().is_file() 
                && entry.path().extension().is_some_and(|e| e == "md")
            {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let name = entry.path()
                            .strip_prefix(root)?
                            .to_string_lossy()
                            .into_owned();
                        files.push((name, modified));
                    }
                }
            }
        }
    }

    // Sort by modified time descending
    files.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top 10
    Ok(files.into_iter().take(10).map(|(name, _)| name).collect())
}

pub fn render(frame: &mut Frame, dashboard: &Dashboard, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Stats panel
    let stats_text = if let Some(stats) = &dashboard.stats {
        format!(
            "Workspace Overview:\n\nTasks: {}\nDependencies: {}\nOrphans: {}\nMax Depth: {}",
            stats.total_tasks,
            stats.total_deps,
            stats.orphaned_tasks,
            stats.max_depth
        )
    } else {
        "Stats unavailable".to_string()
    };
    
    let stats_block = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Stats"))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(stats_block, chunks[0]);

    // Recent files
    let items: Vec<ListItem> = dashboard
        .recent_files
        .iter()
        .map(|f| ListItem::new(f.as_str()))
        .collect();
        
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Recent Activity"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(list, chunks[1]);
}

pub fn handle_key(
    _key: crossterm::event::KeyEvent,
    _dashboard: &mut Dashboard,
) -> anyhow::Result<()> {
    // Dashboard is currently read-only
    Ok(())
}
