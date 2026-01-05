use crate::app::{App, CurrentScreen};
use crate::db::tasks;
use chrono::Datelike;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(target_arch = "wasm32")]
use web_time::Duration;

use once_cell::sync::Lazy;

mod calendar;
pub mod env_manager;

// Lazy-compiled regex for link extraction
static LINK_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?m)(?:^|\s)(?P<label>[^:\n\r]+?)[:：]\s*(?P<url>https?://[^\s]+)").unwrap()
});

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.current_screen {
        CurrentScreen::Menu => render_menu(f, app),
        CurrentScreen::Dashboard => render_dashboard(f, app),
        CurrentScreen::Detail => render_detail(f, app),
        CurrentScreen::Timer => render_timer(f, app),
        CurrentScreen::ReviewDetail => render_review_detail(f, app),
        CurrentScreen::Calendar => calendar::render_calendar(f, app),
        CurrentScreen::ReportEditor => render_editor(f, app),
        CurrentScreen::ReportPreview => render_preview(f, app),
        CurrentScreen::UnifiedMemoList => render_unified_memo_list(f, app),
        CurrentScreen::EnvManager => env_manager::render_env_manager(f, app),
    }

    // Startup Animation: Coalesce (gathering effect)
    let app_time = app.animation.app_start_time.elapsed();
    let startup_duration = Duration::from_millis(800);

    // Manual Reveal Animation (Left to Right)
    // We manually clear the buffer area that hasn't been revealed yet.

    let (elapsed, duration) = if app_time < startup_duration {
        (app_time, startup_duration)
    } else {
        (
            app.animation.last_screen_change.elapsed(),
            Duration::from_millis(500),
        )
    };

    crate::animation::perform_manual_reveal(f, elapsed, duration);
}

fn render_menu(f: &mut Frame, app: &mut App) {
    // 1. Define Main Container Layout
    // Use a single large block with standard single borders
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain) // Standard single line
        .title_top(Line::from(" WorkInfoManage ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Overlay the rotating cyan border animation
    // The animation progress is tracked in app.animation.border_progress
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout (Title, Content, Footer)
    // Reduce area by 1 to fit inside borders
    let inner_area = container_block.inner(main_area);

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header/Subtitle Space
            Constraint::Min(0),    // Menu Items Area
            Constraint::Length(1), // Footer/Help
        ])
        .split(inner_area);

    // 3. Header Section (Subtitle)
    // No borders, just text
    let subtitle = Paragraph::new("Select a feature to start")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(subtitle, parts[0]);

    // 4. Menu Items Section
    let menu_area = parts[1];

    // Add some horizontal padding for the menu
    let menu_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(2),  // Reduced padding (Wider area)
            Constraint::Percentage(96), // Content
            Constraint::Percentage(2),  // Reduced padding
        ])
        .split(menu_area)[1];

    // Render Items
    let menu_entries = vec![
        (
            "Task Manager",
            "Manage tasks with Asana/GitHub sync, Pomodoro timer, work logs.",
        ),
        (
            "Calendar & Reports",
            "View calendar and edit daily reports with markdown support.",
        ),
        (
            "All Memos",
            "Browse all daily reports and task notes in one place.",
        ),
        (
            "Environment Variables",
            "Manage encrypted environment variables securely.",
        ),
    ];

    // Dynamic item height based on available screen space
    // Divide available space by number of items
    let item_count = menu_entries.len();
    // Allow shrinking to 1 line if space is tight to avoid overlap
    let item_height = (menu_area.height / item_count as u16).max(1);

    // Render Sliding Highlight
    let visual_idx = app
        .animation
        .visual_selection
        .value()
        .max(0.0)
        .min((item_count - 1) as f32);

    // Calculate highlight position
    let menu_height = item_height * item_count as u16;
    let menu_top_y = menu_area.y + (menu_area.height.saturating_sub(menu_height) / 2);

    let highlight_y_offset = (visual_idx * item_height as f32).round() as u16;

    // Ensure highlight stays within bounds
    let highlight_y =
        (menu_top_y + highlight_y_offset).min(menu_area.bottom().saturating_sub(item_height));

    let highlight_rect = ratatui::layout::Rect {
        x: menu_area.x,
        y: highlight_y,
        width: menu_area.width,
        height: item_height,
    };

    // Render Highlight Background
    let highlight_block = Block::default()
        .style(Style::default().bg(Color::DarkGray)) // DarkGray for subtle highlight
        .borders(Borders::NONE);
    f.render_widget(highlight_block, highlight_rect);

    // Render Items
    // Menu definition moved up

    for (i, (title, desc)) in menu_entries.iter().enumerate() {
        let y_offset = (i as u16) * item_height;
        let item_y = (menu_top_y + y_offset).min(menu_area.bottom().saturating_sub(item_height));

        let item_rect = ratatui::layout::Rect {
            x: menu_area.x,
            y: item_y,
            width: menu_area.width,
            height: item_height,
        };

        // Determinate style based on selection
        let is_selected = (i as f32 - visual_idx).abs() < 0.5;
        let title_color = if is_selected {
            Color::Cyan
        } else {
            Color::Yellow
        };
        let icon = if is_selected { ">> " } else { "   " };

        let content = vec![
            Line::from(vec![
                Span::styled(icon, Style::default().fg(title_color)),
                Span::styled(
                    *title,
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                format!("   {}", desc),
                Style::default().fg(Color::Gray),
            )),
        ];

        // Add top padding to center the 2-line text
        // If item_height > 2, we can add padding
        let top_padding = if item_height > 2 {
            (item_height - 2) / 2
        } else {
            0
        };

        let p = Paragraph::new(content).block(Block::default().padding(Padding::new(
            2,
            2,
            top_padding,
            0,
        )));

        f.render_widget(p, item_rect);
    }

    // 5. Footer/Help Section
    let help_text = "↑↓/j/k: Navigate | Enter: Select | q: Quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, parts[2]);
}

fn render_editor(f: &mut Frame, app: &mut App) {
    if let Some(ref mut editor) = app.editor_state {
        // 1. Unified Container
        let main_area = f.area();
        let title_text = format!(" Daily Report - {} ", editor.date.format("%Y-%m-%d"));

        let container_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .title_top(Line::from(title_text).alignment(Alignment::Center))
            .style(Style::default());

        f.render_widget(container_block.clone(), main_area);

        // Rotating Border Animation
        crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

        // 2. Inner Area
        let inner_area = container_block.inner(main_area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner_area);

        // Check if we're in input_mode (WASM uses input_buffer instead of TextArea)
        if app.input_mode {
            // Unlikely to be used in TUI but good to keep compatible
            // Render input buffer for WASM with cursor

            // Insert cursor character at cursor position
            let mut display_text = app.input_buffer.clone();
            if app.cursor_position <= display_text.len() {
                display_text.insert(app.cursor_position, '█');
            }

            let input = Paragraph::new(display_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().padding(Padding::uniform(1)))
                .wrap(Wrap { trim: false });
            f.render_widget(input, layout[0]);
        } else {
            // Render TextArea for native
            // Remove the block from textarea so it fits seamlessly
            editor.textarea.remove_block();
            // We might want to add some padding?
            // TextArea doesn't have easy padding without a block.
            // Let's wrap it in a block with padding if needed, or just let it fill.
            // For now, let's try just setting a block with padding but NO borders.
            editor
                .textarea
                .set_block(Block::default().padding(Padding::uniform(1)));
            f.render_widget(&editor.textarea, layout[0]);
        }

        let help_text =
            "Esc: Save & Return | Ctrl+s: Save | Ctrl+c: Cancel | Ctrl+v: Paste | Alt+c: Copy";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(help, layout[1]);
    }
}

fn render_preview(f: &mut Frame, app: &mut App) {
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Report Preview (Coming Soon) ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    let paragraph = Paragraph::new("Preview view will be implemented here.\nPress Esc to return.")
        .alignment(Alignment::Center)
        // Center vertically in the container
        .block(Block::default().padding(Padding::new(
            0,
            0,
            (main_area.height / 2).saturating_sub(1) as u16,
            0,
        )));

    f.render_widget(paragraph, container_block.inner(main_area));
}

fn render_dashboard(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Task Manager ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs (Compact)
            Constraint::Min(2),    // Main content (Clean List)
            Constraint::Length(1), // Footer (Controls)
        ])
        .split(inner_area);

    // Tabs
    let titles: Vec<Line> = ["Development", "Internal Review", "External Review"]
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        // Remove borders from Tabs, just use padding or spacing
        .block(Block::default().padding(Padding::new(0, 0, 0, 1)))
        .select(match app.current_view {
            crate::app::CurrentView::Development => 0,
            crate::app::CurrentView::InternalReview => 1,
            crate::app::CurrentView::ExternalReview => 2,
        })
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Filter tasks based on current view
    // (Existing logic follows, but render to chunks[1])
    let mut rows = Vec::new();

    // Helper closure to create table rows
    let create_table_rows = |status: &str, header: &str, header_color: Color| -> Vec<Row> {
        let mut local_rows = Vec::new();

        let tasks: Vec<(usize, &tasks::Model)> = app
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == status)
            .collect();

        if !tasks.is_empty() {
            // Section Header
            local_rows.push(Row::new(vec![
                Cell::from(""), // Status col filler
                Cell::from(Span::styled(
                    format!("--- {} ---", header),
                    Style::default()
                        .fg(header_color)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(""), // Due col filler
            ]));
        }

        for (idx, t) in tasks {
            let is_selected = idx == app.selected_task_index;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .add_modifier(Modifier::BOLD) // Dark Blue-ish background for selection
            } else {
                Style::default()
            };

            // Due Date Check
            let (due_str, due_fg) = if let Some(due) = t.due_date {
                let s = due.format("%Y-%m-%d").to_string();
                if due < chrono::Local::now().naive_local() {
                    (s, Color::Red)
                } else {
                    (
                        s,
                        if is_selected {
                            Color::White
                        } else {
                            Color::Green
                        },
                    )
                }
            } else {
                ("".to_string(), Color::Gray)
            };

            // Status Badge (Use exact strings as defined in App)
            let (status_text, bg_color) = match t.status.as_str() {
                "Not Started" => ("Not Started", Color::DarkGray),
                "In Progress" => ("In Progress", Color::Blue),
                "Internal Review Checked" => ("Checked", Color::Cyan), // Shorten only if necessary, but user asked for "app defined ones".
                // "Internal Review UnChecked" is long. Let's try to fit it or use a smart abbreviation that is still standard-ish
                // But the user said "Use the ones defined in the app".
                // Let's use the full string but we allocated Constraint::Min(20) or Percentage.
                "Internal Review UnChecked" => ("UnChecked", Color::Red),
                "External Review Checked" => ("Checked", Color::Cyan),
                "External Review UnChecked" => ("UnChecked", Color::Red),
                s => (s, Color::Gray),
            };

            // To respect the user's request for "app defined" but also keep it clean:
            // I will use the raw string if it fits reasonably, or the recognizable suffix for review states if the context is clear from the View.
            // Actually, in the "Internal Review" tab, having "Internal Review UnChecked" is redundant. "UnChecked" is precise enough.
            // Let's stick to the mapped short versions above which are cleaner.

            let status_badge = Span::styled(
                format!(" {} ", status_text),
                Style::default().bg(bg_color).fg(Color::White),
            );

            let title_cell = Cell::from(t.title.as_str());
            let due_cell = Cell::from(Span::styled(due_str, Style::default().fg(due_fg)));
            let status_cell = Cell::from(status_badge);

            local_rows.push(Row::new(vec![status_cell, title_cell, due_cell]).style(style));
        }
        if !local_rows.is_empty() {
            local_rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ])); // Spacing row
        }
        local_rows
    };

    match app.current_view {
        crate::app::CurrentView::Development => {
            rows.extend(create_table_rows(
                "Not Started",
                "Not Started",
                Color::Yellow,
            ));
            rows.extend(create_table_rows(
                "In Progress",
                "In Progress",
                Color::Green,
            ));
        }
        crate::app::CurrentView::InternalReview => {
            rows.extend(create_table_rows(
                "Internal Review UnChecked",
                "UnChecked",
                Color::Red,
            ));
            rows.extend(create_table_rows(
                "Internal Review Checked",
                "Checked",
                Color::Blue,
            ));
        }
        crate::app::CurrentView::ExternalReview => {
            rows.extend(create_table_rows(
                "External Review UnChecked",
                "UnChecked",
                Color::Red,
            ));
            rows.extend(create_table_rows(
                "External Review Checked",
                "Checked",
                Color::Blue,
            ));
        }
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(15), // Wider Status column
            Constraint::Percentage(60),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["Status", "Title", "Due Date"])
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(Block::default().padding(Padding::new(1, 1, 0, 0)));

    f.render_widget(table, chunks[1]);

    // Footer lines
    let status = if app.timer.start_time.is_some() {
        "[TIMER RUNNING]"
    } else {
        ""
    };
    let footer_text = format!(
        "q: Quit | Tab: View | Enter: Detail | n: New | c: Complete | {} {}",
        app.status_message, status
    );

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

fn render_detail(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Task Detail ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),    // Detail Content (Scrollable)
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    if let Some(selected_task) = app.tasks.get(app.selected_task_index) {
        let mut detail_lines = vec![
            Line::from(vec![
                Span::styled(
                    "Title: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &selected_task.title,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Asana ID: ", Style::default().fg(Color::Yellow)),
                Span::raw(&selected_task.asana_id),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &selected_task.status,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        // Extract and display task type from description (reused logic)
        if let Some(desc) = &selected_task.description {
            if let Some(start) = desc.find("[Type: ") {
                if let Some(end) = desc[start..].find("]") {
                    let task_type = &desc[start + 7..start + end];
                    detail_lines.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().fg(Color::Yellow)),
                        Span::styled(task_type, Style::default().fg(Color::Magenta)),
                    ]));
                }
            }
        }

        if let Some(due_date) = selected_task.due_date {
            let today = chrono::Local::now().naive_local();
            let is_overdue = due_date < today;
            let due_str = due_date.format("%Y-%m-%d").to_string();
            let due_color = if is_overdue { Color::Red } else { Color::Green };
            let prefix = if is_overdue {
                "⚠ Due Date (OVERDUE): "
            } else {
                "Due Date: "
            };

            detail_lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(
                    due_str,
                    Style::default().fg(due_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            detail_lines.push(Line::from(vec![
                Span::styled("Due Date: ", Style::default().fg(Color::Yellow)),
                Span::raw("Not set"),
            ]));
        }

        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![Span::styled(
            "Description: ",
            Style::default().fg(Color::Yellow),
        )]));

        // Description Logic
        let desc_text = selected_task.description.as_deref().unwrap_or("N/A");
        let first_line = desc_text
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect::<String>();
        let display_text = if desc_text.chars().count() > 60 || desc_text.lines().count() > 1 {
            format!("{}...", first_line)
        } else {
            first_line
        };
        detail_lines.push(Line::from(Span::raw(display_text)));

        // Extract Links
        if let Some(desc) = &selected_task.description {
            for cap in LINK_REGEX.captures_iter(desc) {
                if let (Some(label), Some(url)) = (cap.name("label"), cap.name("url")) {
                    // Check if not empty
                    if !detail_lines
                        .last()
                        .unwrap()
                        .spans
                        .first()
                        .map(|s| s.content == "--- Resources / Links ---")
                        .unwrap_or(false)
                    {
                        detail_lines.push(Line::from(""));
                        detail_lines.push(Line::from(vec![Span::styled(
                            "--- Resources / Links ---",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )]));
                    }
                    detail_lines.push(Line::from(vec![
                        Span::styled(
                            format!("[{}] ", label.as_str().trim()),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::styled(
                            url.as_str(),
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                    ]));
                }
            }
        }

        // Display Notes grouped by Date
        if let Some(notes) = app.notes.get(&selected_task.id) {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![Span::styled(
                "--- Memos ---",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )]));

            // Group by date
            let mut temp_map: std::collections::BTreeMap<
                chrono::NaiveDate,
                Vec<&crate::db::task_notes::Model>,
            > = std::collections::BTreeMap::new();

            for note in notes {
                let date = note.created_at.date();
                temp_map.entry(date).or_default().push(note);
            }

            // Iterate BTreeMap directly (it's sorted by key ascending, so rev() gives newest first)
            for (date, mut date_notes) in temp_map.into_iter().rev() {
                date_notes.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Sort notes within day desc

                detail_lines.push(Line::from(vec![Span::styled(
                    format!("--- {} ---", date.format("%Y-%m-%d")),
                    Style::default().fg(Color::DarkGray),
                )]));

                for note in date_notes {
                    detail_lines.push(Line::from(vec![
                        Span::styled(
                            format!("  [{}] ", note.created_at.format("%H:%M")),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::raw(&note.content),
                    ]));
                }
            }
        }

        let details = Paragraph::new(detail_lines)
            .block(Block::default().padding(Padding::new(1, 1, 0, 0))) // Remove borders, add padding
            .wrap(Wrap { trim: true });
        f.render_widget(details, chunks[0]);
    }

    let footer_text = "Esc: Back | n: Add Memo | r: Review Status | t: Toggle Timer";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[1]);

    // Input Popup - TextArea for Markdown editing
    if app.input_mode {
        if let Some(ref mut textarea) = app.task_note_textarea {
            let area = centered_rect(80, 60, f.area());
            f.render_widget(ratatui::widgets::Clear, area); // Clear background

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(area);

            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add Note (Markdown)")
                    .style(Style::default().fg(Color::Cyan)),
            );
            f.render_widget(&*textarea, layout[0]);

            let help = Paragraph::new("Esc: Save & Close | Ctrl+c: Cancel | Ctrl+v: Paste | Alt+c: Copy | Search highlights headers & bold")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(help, layout[1]);
        } else {
            // Fallback to simple input buffer with cursor
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Add Memo")
                .style(Style::default().bg(Color::Blue).fg(Color::White));

            let area = centered_rect(60, 20, f.area());
            f.render_widget(ratatui::widgets::Clear, area); // Clear background

            // Insert cursor character at cursor position
            let mut display_text = app.input_buffer.clone();
            if app.cursor_position <= display_text.len() {
                display_text.insert(app.cursor_position, '█');
            }

            let input = Paragraph::new(display_text)
                .style(Style::default().fg(Color::Yellow))
                .block(block);
            f.render_widget(input, area);
        }
    }
}

fn render_timer(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Pomodoro Timer ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),    // Timer Display
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    let timer_text = if let Some(start_time) = app.timer.start_time {
        let now = chrono::Local::now();
        let elapsed = now.signed_duration_since(start_time).num_seconds();
        let minutes = elapsed / 60;
        let seconds = elapsed % 60;

        vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("{:02}:{:02}", minutes, seconds),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Focusing...",
                Style::default().fg(Color::Cyan),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "00:00",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Ready to start",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let p = Paragraph::new(timer_text)
        .alignment(Alignment::Center)
        // No borders on paragraph
        .block(Block::default().padding(Padding::new(0, 0, (inner_area.height / 3) as u16, 0))); // Vertically center approx

    f.render_widget(p, chunks[0]);

    let footer_text = "Esc: Back | Space: Start/Stop | r: Reset";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[1]);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_review_detail(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Review Details ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    if let Some(selected_task) = app.tasks.get(app.selected_task_index) {
        // Parse Review Status
        let mut rows = Vec::new();
        if let Some(json_val) = &selected_task.review_status {
            use crate::logic::sync::TaskReviewStatus;

            if let Ok(status) = serde_json::from_value::<TaskReviewStatus>(json_val.clone()) {
                // Internal
                rows.push(
                    Row::new(vec![
                        Cell::from("INTERNAL REVIEWERS"),
                        Cell::from(""),
                        Cell::from(""),
                    ])
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
                for r in status.internal {
                    let color = match r.status.as_str() {
                        "Approved" => Color::Green,
                        "Changes Requested" => Color::Red,
                        "Commented" => Color::Blue,
                        "Pending" => Color::DarkGray,
                        _ => Color::White,
                    };
                    rows.push(Row::new(vec![
                        Cell::from(r.name),
                        Cell::from(Span::styled(r.status, Style::default().fg(color))),
                        Cell::from(r.url.unwrap_or_default()),
                    ]));
                }

                rows.push(Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])); // Spacer

                // External
                rows.push(
                    Row::new(vec![
                        Cell::from("EXTERNAL REVIEWERS"),
                        Cell::from(""),
                        Cell::from(""),
                    ])
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
                for r in status.external {
                    let color = match r.status.as_str() {
                        "Approved" => Color::Green,
                        "Changes Requested" => Color::Red,
                        "Commented" => Color::Blue,
                        "Pending" => Color::DarkGray,
                        _ => Color::White,
                    };
                    rows.push(Row::new(vec![
                        Cell::from(r.name),
                        Cell::from(Span::styled(r.status, Style::default().fg(color))),
                        Cell::from(r.url.unwrap_or_default()),
                    ]));
                }
            } else {
                rows.push(Row::new(vec![
                    Cell::from("Failed to parse review data"),
                    Cell::from(""),
                    Cell::from(""),
                ]));
            }
        } else {
            rows.push(Row::new(vec![
                Cell::from("No review data available"),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40),
            ],
        )
        .block(Block::default().padding(Padding::new(1, 1, 0, 0)))
        .header(
            Row::new(vec!["Reviewer", "Status", "URL"])
                .style(Style::default().fg(Color::Cyan))
                .bottom_margin(1),
        )
        .column_spacing(1);

        f.render_widget(table, chunks[0]);
    }

    // Footer
    let footer_text = "Esc: Back | Enter: Detail";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[1]);
}

fn render_unified_memo_list(f: &mut Frame, app: &mut App) {
    // 1. Unified Container
    let main_area = f.area();
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .title_top(Line::from(" Unified Memo List ").alignment(Alignment::Center))
        .style(Style::default());

    f.render_widget(container_block.clone(), main_area);

    // Rotating Border Animation
    crate::animation::draw_traveling_border(f, main_area, app.animation.border_progress);

    // 2. Inner Layout
    let inner_area = container_block.inner(main_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Memo list (Tree)
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    if let Some(ref mut state) = app.unified_memo_list_state {
        // Group items by hierarchy
        // Structure: Year -> Month -> Day -> Leaf

        let mut root_items: Vec<TreeItem<'static, String>> = Vec::new(); // Changed to 'static lifetime for owned strings

        // We actually need to reconstruct the tree structure every frame since TreeItems hold references.
        // Grouping logic:
        let mut hierarchy: std::collections::BTreeMap<
            i32,
            std::collections::BTreeMap<
                u32,
                std::collections::BTreeMap<chrono::NaiveDate, Vec<&crate::app::UnifiedMemoItem>>,
            >,
        > = std::collections::BTreeMap::new();

        for item in &state.items {
            let date = item.date();
            let year = date.year();
            let month = date.month();

            hierarchy
                .entry(year)
                .or_default()
                .entry(month)
                .or_default()
                .entry(date)
                .or_default()
                .push(item);
        }

        for (year, months) in hierarchy.iter().rev() {
            let mut year_items = Vec::new();
            for (month, days) in months.iter().rev() {
                let mut month_items = Vec::new();
                for (date, memos) in days.iter().rev() {
                    let mut day_items = Vec::new();
                    for memo in memos {
                        let (time_str, preview_text) = match memo {
                            crate::app::UnifiedMemoItem::DailyReport { title, .. } => {
                                ("09:00".to_string(), title.clone())
                            } // Default time for report
                            crate::app::UnifiedMemoItem::TaskNote {
                                created_at,
                                task_title,
                                ..
                            } => (
                                created_at.format("%H:%M").to_string(),
                                format!("📝 {}", task_title), // Simple preview
                            ),
                        };

                        day_items.push(TreeItem::new_leaf(
                            memo.id(),
                            Line::from(vec![
                                Span::styled(
                                    format!("[{}] ", time_str),
                                    Style::default().fg(Color::DarkGray),
                                ),
                                Span::raw(preview_text),
                            ]),
                        ));
                    }
                    month_items.push(
                        TreeItem::new(
                            date.to_string(),
                            Line::from(vec![
                                Span::styled(
                                    format!("{} ", date.format("%d (%a)")),
                                    Style::default().fg(Color::Cyan),
                                ),
                                Span::styled(
                                    format!("({})", memos.len()),
                                    Style::default().fg(Color::DarkGray),
                                ),
                            ]),
                            day_items,
                        )
                        .expect("Duplicate ID"),
                    );
                }
                year_items.push(
                    TreeItem::new(
                        month.to_string(),
                        Line::from(format!(
                            "{}",
                            chrono::Month::try_from(*month as u8).unwrap().name()
                        )),
                        month_items,
                    )
                    .expect("Duplicate ID"),
                );
            }
            root_items.push(
                TreeItem::new(
                    year.to_string(),
                    Line::from(format!("{}", year)),
                    year_items,
                )
                .expect("Duplicate ID"),
            );
        }

        let tree_widget = Tree::new(&root_items)
            .expect("Failed to create tree widget")
            .block(Block::default().padding(Padding::new(1, 1, 0, 0))) // Remove borders
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(tree_widget, chunks[0], &mut state.tree_state);

        // Render footer in stateful block too
        let footer_text = "Esc: Back | Space: Toggle | Enter: View | r: Reset";
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[1]);
    } else {
        let p = Paragraph::new("No memos available")
            .alignment(Alignment::Center)
            // No borders on paragraph
            .block(Block::default().padding(Padding::new(0, 0, (chunks[0].height / 3) as u16, 0))); // Vertically center approx

        f.render_widget(p, chunks[0]);

        let footer_text = "Esc: Back | Space: Toggle | Enter: View | r: Reset";
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[1]);
    }
}
