use ratatui::{
    layout::{Constraint, Rect, Layout, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Cell, Row, Table},
    Frame,
};
use chrono::{Datelike, NaiveDate, Duration};
use crate::app::{App, CalendarState};

pub fn render_calendar(f: &mut Frame, app: &mut App) {
    let state = match &app.calendar_state {
        Some(s) => s,
        None => return,
    };

    render_calendar_grid(f, f.area(), state);
}

fn render_calendar_grid(f: &mut Frame, area: Rect, state: &CalendarState) {
    // Create title with month, year, and help text
    let month_name = get_month_name(state.current_month.month());
    let title = format!(
        "📅 {} {} | [/]: Month | ←↓↑→/hjkl: Move | ⏎: Edit | ⇧⇥: Cycle | ESC: Return",
        month_name,
        state.current_month.year()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Layout: Header (Weekdays) + Grid
    // We'll use a Table for the grid, including the header.
    
    // Calculate grid start date (Sunday of the first week of the month view)
    // We want a 6-week view to be safe.
    let first_day_of_month = state.current_month.with_day(1).unwrap_or(state.current_month);
    let days_from_sunday = first_day_of_month.weekday().num_days_from_sunday(); // 0 for Sunday
    let grid_start_date = first_day_of_month - Duration::days(days_from_sunday as i64);

    // Calculate dimensions
    // We have inner_area.height available.
    // Header takes 1 line (+1 for spacing/border if we want, but simple header is 1 line).
    // We want 6 rows.
    let header_height = 1;
    let available_height = inner_area.height.saturating_sub(header_height);
    let row_height = (available_height / 6).max(1);
    
    // Create Header Row
    let header_cells = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(header_height);

    // Create Calendar Rows
    let mut rows = Vec::new();
    let mut current_grid_date = grid_start_date;
    let today = chrono::Local::now().naive_local().date();

    for _ in 0..6 {
        let mut row_cells = Vec::new();
        for _ in 0..7 {
            // Styling logic
            let is_today = current_grid_date == today;
            let is_selected = current_grid_date == state.selected_date;
            let is_current_month = current_grid_date.month() == state.current_month.month();
            let has_report = state.report_dates.contains(&current_grid_date);

            let mut style = Style::default();

            // Base color depends on whether it's current month
            if is_current_month {
                style = style.fg(Color::White);
            } else {
                style = style.fg(Color::DarkGray);
            }

            // Report marker
            if has_report {
                style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
            }

            // Today highlight
            if is_today {
                style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
            }

            // Selection override
            if is_selected {
                style = style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
            }

            let cell_content = format!("{}", current_grid_date.day());
            row_cells.push(Cell::from(cell_content).style(style));
            
            current_grid_date += Duration::days(1);
        }
        rows.push(Row::new(row_cells).height(row_height));
    }

    let widths = [
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
        Constraint::Ratio(1, 7),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(0); // Compact grid look

    f.render_widget(table, inner_area);
}

fn get_month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
