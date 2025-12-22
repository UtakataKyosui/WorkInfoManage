use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Table, Row, Cell, Tabs, List, ListItem},
    Frame,
};
use crate::app::{App, CurrentScreen};
use crate::db::tasks;

use once_cell::sync::Lazy;

mod calendar;

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
    }
}

fn render_menu(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Min(0),      // Menu items
            Constraint::Length(3),   // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("WorkInfoManage", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - Select a feature to start"),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu items using List widget
    let items = vec![
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("Task Manager", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  Manage tasks with Asana/GitHub sync, Pomodoro timer, work logs, and markdown notes"),
        ]),
        ListItem::new(vec![
            Line::from(vec![
                Span::styled("Calendar & Daily Reports", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  View calendar and edit daily reports with markdown editor support"),
        ]),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Features"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut ratatui::widgets::ListState::default().with_selected(Some(app.menu_selection)));

    // Help
    let help = Paragraph::new("↑↓/j/k: Navigate | Enter: Select | q: Quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn render_editor(f: &mut Frame, app: &mut App) {
    if let Some(ref mut editor) = app.editor_state {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        editor.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Daily Report - {}", editor.date.format("%Y-%m-%d")))
                .style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(&editor.textarea, layout[0]);

        let help = Paragraph::new("Esc: Save & Return | Ctrl+s: Save | Ctrl+c: Cancel | Ctrl+v: Paste | Alt+c: Copy | Search highlights headers & bold")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(help, layout[1]);
    }
}

fn render_preview(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Report Preview (Coming Soon)")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new("Preview view will be implemented here.\nPress Esc to return.")
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, f.area());
}

fn render_dashboard(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),      // Title
                Constraint::Length(3),      // Tabs
                Constraint::Min(2),         // Main content (Clean List)
                Constraint::Length(4),      // Footer (Controls)
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("TaskManager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Tabs
    let titles: Vec<Line> = vec!["Development", "Internal Review", "External Review"]
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
        .block(Block::default().borders(Borders::ALL).title("Views (1-3 or Tab)"))
        .select(match app.current_view {
            crate::app::CurrentView::Development => 0,
            crate::app::CurrentView::InternalReview => 1,
            crate::app::CurrentView::ExternalReview => 2,
        })
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[1]);
    
    // Filter tasks based on current view
    let mut rows = Vec::new();

    // Helper closure to create table rows
    let create_table_rows = |status: &str, header: &str, header_color: Color| -> Vec<Row> {
        let mut local_rows = Vec::new();
        
        let tasks: Vec<(usize, &tasks::Model)> = app.tasks.iter()
            .enumerate()
            .filter(|(_, t)| t.status == status)
            .collect();
            
        if !tasks.is_empty() {
             // Section Header
             local_rows.push(Row::new(vec![
                 Cell::from(Span::styled(format!("--- {} ---", header), Style::default().fg(header_color).add_modifier(Modifier::BOLD))),
                 Cell::from(""),
             ]));
        }
        
        for (idx, t) in tasks {
            let is_selected = idx == app.selected_task_index;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            // Due Date Check
            let (due_str, due_fg) = if let Some(due) = t.due_date {
                let s = due.format("%Y-%m-%d").to_string();
                if due < chrono::Local::now().naive_local() {
                    (s, Color::Red)
                } else {
                    (s, if is_selected { Color::White } else { Color::Green })
                }
            } else {
                ("".to_string(), Color::Gray)
            };

            let title_cell = Cell::from(t.title.as_str());
            let due_cell = Cell::from(Span::styled(due_str, Style::default().fg(due_fg)));
            
            local_rows.push(Row::new(vec![title_cell, due_cell]).style(style));
        }
        if !local_rows.is_empty() {
             local_rows.push(Row::new(vec![Cell::from(""), Cell::from("")])); // Spacing row
        }
        local_rows
    };

    match app.current_view {
        crate::app::CurrentView::Development => {
            rows.extend(create_table_rows("Not Started", "Not Started", Color::Yellow));
            rows.extend(create_table_rows("In Progress", "In Progress", Color::Green));
        },
        crate::app::CurrentView::InternalReview => {
            rows.extend(create_table_rows("Internal Review UnChecked", "UnChecked", Color::Red));
            rows.extend(create_table_rows("Internal Review Checked", "Checked", Color::Blue));
        },
        crate::app::CurrentView::ExternalReview => {
            rows.extend(create_table_rows("External Review UnChecked", "UnChecked", Color::Red));
            rows.extend(create_table_rows("External Review Checked", "Checked", Color::Blue));
        }
    };

    let title = match app.current_view {
        crate::app::CurrentView::Development => "Tasks",
        crate::app::CurrentView::InternalReview => "Tasks",
        crate::app::CurrentView::ExternalReview => "Tasks",
    };

    let table = Table::new(
        rows,
        [Constraint::Percentage(80), Constraint::Length(12)]
    )
    .header(Row::new(vec!["Title", "Due Date"]).style(Style::default().add_modifier(Modifier::UNDERLINED)))
    .block(Block::default().borders(Borders::ALL).title(title));
    
    f.render_widget(table, chunks[2]);
    
    // Status/Footer
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(chunks[3]);

    let footer_text = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Enter: Detail | "),
            Span::raw("T: Timer | "),
            Span::raw("t: Toggle | "),
            Span::raw("1-3/Tab: View | "),
            Span::styled("Esc/q: Quit", Style::default().fg(Color::Red)),
        ]),
         Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(&app.status_message),
             if let Some(_) = app.timer.start_time {
                Span::styled(" [TIMER RUNNING]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("")
            }
        ]),
    ];
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, footer_chunks[0]);

    // Sync Status
    let sync_text = if let Some(last_sync) = app.sync_state.last_sync_time {
        let status_color = if app.sync_state.error.is_some() { Color::Red } else { Color::Green };
        let status_text = if app.sync_state.error.is_some() { "ERR" } else { "OK" };
        
        vec![
            Line::from(vec![
                Span::styled("Last: ", Style::default().fg(Color::Cyan)),
                Span::raw(last_sync.format("%H:%M:%S").to_string()),
                Span::styled(format!(" [{}]", status_text), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Tasks: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}", app.sync_state.total_tasks)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("Not synced yet", Style::default().fg(Color::Gray)))]
    };
    
    let sync_block = Paragraph::new(sync_text)
        .block(Block::default().borders(Borders::ALL).title("Sync Status"));
    f.render_widget(sync_block, footer_chunks[1]);
}

fn render_detail(f: &mut Frame, app: &mut App) {
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(2),         // Detail Content
                Constraint::Length(3),      // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    if let Some(selected_task) = app.tasks.get(app.selected_task_index) {
        let mut detail_lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(&selected_task.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Asana ID: ", Style::default().fg(Color::Yellow)),
                Span::raw(&selected_task.asana_id),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(&selected_task.status, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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
            let prefix = if is_overdue { "⚠ Due Date (OVERDUE): " } else { "Due Date: " };
            
            detail_lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(due_str, Style::default().fg(due_color).add_modifier(Modifier::BOLD)),
            ]));
        } else {
             detail_lines.push(Line::from(vec![
                Span::styled("Due Date: ", Style::default().fg(Color::Yellow)),
                Span::raw("Not set"),
            ]));
        }

        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Yellow)),
        ]));
        
        
        let desc_text = selected_task.description.as_deref().unwrap_or("N/A");
        let first_line = desc_text.lines().next().unwrap_or("").chars().take(60).collect::<String>();
        let display_text = if desc_text.chars().count() > 60 || desc_text.lines().count() > 1 {
            format!("{}...", first_line)
        } else {
            first_line
        };
        detail_lines.push(Line::from(Span::raw(display_text)));

        // Extract Links
        if let Some(desc) = &selected_task.description {
            // Use pre-compiled regex for link extraction
            let mut links = Vec::new();
            
            for cap in LINK_REGEX.captures_iter(desc) {
                if let (Some(label), Some(url)) = (cap.name("label"), cap.name("url")) {
                     links.push((label.as_str().trim().to_string(), url.as_str().to_string()));
                }
            }
            
            if !links.is_empty() {
                detail_lines.push(Line::from(""));
                detail_lines.push(Line::from(vec![
                    Span::styled("--- Resources / Links ---", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
                
                for (label, link) in links {
                    detail_lines.push(Line::from(vec![
                        Span::styled(format!("[{}] ", label), Style::default().fg(Color::Magenta)),
                        Span::styled(link, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
                    ]));
                }
            }
        }

        // Display Notes
        if let Some(notes) = app.notes.get(&selected_task.id) {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("--- Memos ---", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]));
            for note in notes {
                detail_lines.push(Line::from(vec![
                    Span::styled(format!("[{}] ", note.created_at.format("%m/%d %H:%M")), Style::default().fg(Color::DarkGray)),
                    Span::raw(&note.content),
                ]));
            }
        }
        
        let details = Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title("Task Details"))
            .wrap(Wrap { trim: true });
        f.render_widget(details, chunks[0]);
    }

    let footer_text = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Esc: Back | "),
            Span::raw("n: Add Memo | "),
            Span::raw("r: Review Status | "),
            Span::raw("t: Toggle Timer"),
        ]),
    ];
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
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
            // Fallback to simple input buffer
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Add Memo")
                .style(Style::default().bg(Color::Blue).fg(Color::White));

            let area = centered_rect(60, 20, f.area());
            f.render_widget(ratatui::widgets::Clear, area); // Clear background

            let input = Paragraph::new(app.input_buffer.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(block);
            f.render_widget(input, area);
        }
    }
}

fn render_timer(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(2),         // Timer Display
                Constraint::Length(3),      // Footer
            ]
            .as_ref(),
        )
        .split(f.area());
        
    let timer_text = if let Some(start_time) = app.timer.start_time {
        let now = chrono::Local::now();
        let elapsed = now.signed_duration_since(start_time).num_seconds();
        let minutes = elapsed / 60;
        let seconds = elapsed % 60;
        
        // Large ASCII art style text could go here, but for now just big text
        vec![
            Line::from(""),
            Line::from(Span::styled(format!("{:02}:{:02}", minutes, seconds), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC))),
             Line::from(""),
             Line::from(format!("Cycle: {}", app.timer.cycle_count)),
             Line::from(""),
             Line::from(if let Some(task_id) = app.timer.active_task_id {
                 if let Some(task) = app.tasks.iter().find(|t| t.id == task_id) {
                     format!("Working on: {}", task.title)
                 } else {
                     "Unknown Task".to_string()
                 }
             } else {
                 "No Task Selected".to_string()
             })
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled("Timer Stopped", Style::default().fg(Color::Gray))),
        ]
    };
    
    let timer_display = Paragraph::new(timer_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Timer"));
    f.render_widget(timer_display, chunks[0]);
    
    let footer_text = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
             Span::raw("Esc: Back | "),
            Span::raw("t: Toggle Timer"),
        ]),
    ];
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),      // Header
                Constraint::Min(2),         // Content
                Constraint::Length(3),      // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    if let Some(selected_task) = app.tasks.get(app.selected_task_index) {
        // Title
        let title_text = format!("Review Status: {}", selected_task.title);
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);
        
        // Parse Review Status
        let mut rows = Vec::new();
        if let Some(json_val) = &selected_task.review_status {
             use crate::logic::sync::TaskReviewStatus;
             
             if let Ok(status) = serde_json::from_value::<TaskReviewStatus>(json_val.clone()) {
                 
                 // Internal
                 rows.push(Row::new(vec![Cell::from("INTERNAL REVIEWERS"), Cell::from(""), Cell::from("")]).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
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
                 
                 rows.push(Row::new(vec![Cell::from(""), Cell::from(""), Cell::from("")])); // Spacer
                 
                 // External
                 rows.push(Row::new(vec![Cell::from("EXTERNAL REVIEWERS"), Cell::from(""), Cell::from("")]).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
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
                 rows.push(Row::new(vec![Cell::from("Failed to parse review data"), Cell::from(""), Cell::from("")]));
             }
        } else {
             rows.push(Row::new(vec![Cell::from("No review data available"), Cell::from(""), Cell::from("")]));
        }
        
        let table = Table::new(rows, [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ])
        .header(Row::new(vec!["Reviewer", "Status", "Link"]).style(Style::default().add_modifier(Modifier::UNDERLINED)))
        .block(Block::default().borders(Borders::ALL).title("Review Details"));
        
        f.render_widget(table, chunks[1]);
    }
    
    // Footer
    let footer_text = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Esc: Back | "),
            Span::raw("Enter: Detail"),
        ]),
    ];
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, chunks[2]);
}
