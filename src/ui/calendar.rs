use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    Frame,
};
use chrono::{Datelike, NaiveDate, Months};
use crate::app::{App, CalendarState};

pub fn render_calendar(f: &mut Frame, app: &mut App) {
    let state = match &app.calendar_state {
        Some(s) => s,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Calendar
            Constraint::Length(3),  // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(format!(
        "Calendar - {} {}",
        get_month_name(state.current_month.month()),
        state.current_month.year()
    ))
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Calendar grid
    render_calendar_grid(f, chunks[1], state);

    // Help text
    let help = Paragraph::new("← →: Change month | ↑↓: Move week | Enter: Edit report | m: Memos | Shift+Tab: Back to tasks | Esc: Exit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn render_calendar_grid(f: &mut Frame, area: Rect, state: &CalendarState) {
    let first_day = NaiveDate::from_ymd_opt(
        state.current_month.year(),
        state.current_month.month(),
        1
    ).unwrap();
    
    let days_in_month = get_days_in_month(state.current_month.year(), state.current_month.month());
    let first_weekday = first_day.weekday().num_days_from_monday() as usize;

    // Header row
    let header = Row::new(vec![
        Cell::from("Mon").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Tue").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Wed").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Thu").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Fri").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Sat").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Sun").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]).height(1);

    // Calendar rows
    let mut rows = vec![header];
    let mut week = vec![Cell::from(""); 7];
    let mut day_index = 0;

    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(
            state.current_month.year(),
            state.current_month.month(),
            day
        ).unwrap();

        let col = (first_weekday + day_index) % 7;
        
        let mut style = Style::default();
        
        // Highlight selected date
        if date == state.selected_date {
            style = style.bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD);
        }
        // Highlight today
        else if date == chrono::Local::now().naive_local().date() {
            style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
        }
        // Highlight dates with reports
        else if state.report_dates.contains(&date) {
            style = style.fg(Color::Cyan);
        }

        week[col] = Cell::from(format!("{:2}", day)).style(style);

        if col == 6 {
            rows.push(Row::new(week.clone()).height(1));
            week = vec![Cell::from(""); 7];
        }
        
        day_index += 1;
    }

    // Add last incomplete week if needed
    if day_index % 7 != 0 {
        rows.push(Row::new(week).height(1));
    }

    let widths = [Constraint::Percentage(14); 7];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::ALL).title("Calendar"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn get_days_in_month(year: i32, month: u32) -> u32 {
    // Get the first day of the month, add one month, then subtract one day
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid year or month");
    let next_month_first = first_day.checked_add_months(Months::new(1)).expect("Could not determine next month");
    next_month_first.pred_opt().expect("Could not determine previous day").day()
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
