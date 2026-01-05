use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render_env_manager(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Environment Variables ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Variable List
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    if let Some(ref manager) = app.env_manager {
        let rows: Vec<Row> = manager
            .variables
            .iter()
            .enumerate()
            .map(|(i, var)| {
                let is_selected = i == manager.selection;

                if is_selected && app.input_mode {
                    // Inline Editing Mode
                    let mut display_text = app.input_buffer.clone();
                    if app.cursor_position <= display_text.len() {
                        display_text.insert(app.cursor_position, '█');
                    }

                    Row::new(vec![
                        Cell::from(var.key.clone()).style(Style::default().fg(Color::Yellow)),
                        Cell::from(Span::styled(
                            display_text,
                            Style::default().bg(Color::Blue).fg(Color::White),
                        )),
                    ])
                    .style(Style::default().add_modifier(Modifier::BOLD))
                } else {
                    let style = if is_selected {
                        Style::default()
                            .bg(Color::Rgb(50, 50, 80))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let value_display = if var.value.is_empty() {
                        Span::styled("<empty>", Style::default().fg(Color::DarkGray))
                    } else {
                        Span::styled("********", Style::default().fg(Color::Gray))
                    };

                    Row::new(vec![Cell::from(var.key.clone()), Cell::from(value_display)])
                        .style(style)
                }
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(40), Constraint::Percentage(60)],
        )
        .header(
            Row::new(vec!["Key", "Value"])
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::NONE));

        f.render_widget(table, chunks[0]);
    } else {
        let p = Paragraph::new("Initializing Environment Manager...").alignment(Alignment::Center);
        f.render_widget(p, chunks[0]);
    }

    // Footer
    let mut footer_spans = vec![Span::styled(
        "Esc: Back | ↑↓: Navigate | Enter: Edit | s: Save",
        Style::default().fg(Color::DarkGray),
    )];

    if !app.status_message.is_empty() {
        footer_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        let style = if app.status_message.to_lowercase().contains("saved") {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        footer_spans.push(Span::styled(&app.status_message, style));
    }

    let footer = Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);

    // Input overlay removed in favor of inline editing
}
