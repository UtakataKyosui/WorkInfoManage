use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Storage Type Tabs
            Constraint::Min(5),    // Content (URL input etc)
            Constraint::Length(3), // Footer/Status
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Text::styled(
        "Configuration Manager",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if let Some(config_manager) = &app.config_manager {
        // Storage Type Tabs
        let tabs = Tabs::new(vec!["1. JSON File", "2. Database"])
            .block(
                Block::default()
                    .title("Storage Backend")
                    .borders(Borders::ALL),
            )
            .select(if config_manager.is_database() { 1 } else { 0 })
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[1]);

        // Content Area
        let content_block = Block::default().borders(Borders::ALL).title("Settings");
        let content_area = content_block.inner(chunks[2]);
        f.render_widget(content_block, chunks[2]);

        if config_manager.is_database() {
            let db_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3)])
                .split(content_area);

            let url = config_manager.get_database_url();

            // If in input mode and editing URL, show cursor?
            // The App struct handles input_buffer. We need to sync buffer with url when entering this mode?
            // For now, let's display the current value.

            let display_text = if app.input_mode {
                format!("{} (Editing...)", app.input_buffer)
            } else {
                url
            };

            let input = Paragraph::new(display_text)
                .block(
                    Block::default()
                        .title("Database URL (Press Enter to Edit)")
                        .borders(Borders::ALL),
                )
                .style(if app.input_mode {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                });
            f.render_widget(input, db_chunks[0]);
        } else {
            let info =
                Paragraph::new("JSON storage uses local files.\nPath: ~/task-manage/data.json")
                    .style(Style::default().fg(Color::Gray));
            f.render_widget(info, content_area);
        }

        // Status Footer
        let help_text = "Tab: Switch | Enter: Edit | Ctrl+s: Save | q: Quit";
        let status_content = vec![
            Line::from(config_manager.status_message.as_str()),
            Line::from(""),
            Line::from(Span::styled(help_text, Style::default().fg(Color::Gray))),
        ];

        let status = Paragraph::new(status_content)
            .style(if config_manager.status_message.contains("saved") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            })
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, chunks[3]);
    }
}
