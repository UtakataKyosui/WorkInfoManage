use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, BorderType, Padding, calendar::{Monthly, CalendarEventStore}},
    Frame,
};
use chrono::{Datelike, NaiveDate};
use time::{Date, Month as TimeMonth};
use crate::app::{App, CalendarState};

pub fn render_calendar(f: &mut Frame, app: &mut App) {
    let state = match &app.calendar_state {
        Some(s) => s,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Calendar (takes most space)
            Constraint::Length(3),  // Help
        ])
        .split(f.area());

    // Calendar grid with improved design - title will be in the calendar block
    render_calendar_grid(f, chunks[0], state);

    // Help text with icons - brighter color for visibility
    let help = Paragraph::new("◄ ►: Month | ▲ ▼: Week | ⏎: Edit | ⇧⇥: Tasks | ESC: Menu")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray)));
    f.render_widget(help, chunks[1]);
}

fn render_calendar_grid(f: &mut Frame, area: Rect, state: &CalendarState) {
    // Convert chrono NaiveDate to time::Date
    let current_date = chrono_to_time_date(state.current_month);

    // Create event store for marking dates
    let mut event_store = CalendarEventStore::default();

    // Add report dates to the event store
    for report_date in &state.report_dates {
        let time_date = chrono_to_time_date(*report_date);
        event_store.add(time_date, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    }

    // Highlight selected date
    let selected_time_date = chrono_to_time_date(state.selected_date);
    event_store.add(selected_time_date, Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD));

    // Highlight today
    let today = chrono::Local::now().naive_local().date();
    let today_time_date = chrono_to_time_date(today);
    if !state.report_dates.contains(&today) && today != state.selected_date {
        event_store.add(today_time_date, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    }

    // Create title with month and year
    let title = format!(
        "📅 Calendar - {} {}",
        get_month_name(state.current_month.month()),
        state.current_month.year()
    );

    // Create the calendar widget with padding for better spacing
    let calendar = Monthly::new(current_date, event_store)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::new(3, 3, 2, 2)))  // left, right, top, bottom padding - increased for larger size
        .show_month_header(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .show_weekdays_header(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .show_surrounding(Style::default().fg(Color::DarkGray));  // Show surrounding month days

    f.render_widget(calendar, area);
}

fn chrono_to_time_date(date: NaiveDate) -> Date {
    let month = match date.month() {
        1 => TimeMonth::January,
        2 => TimeMonth::February,
        3 => TimeMonth::March,
        4 => TimeMonth::April,
        5 => TimeMonth::May,
        6 => TimeMonth::June,
        7 => TimeMonth::July,
        8 => TimeMonth::August,
        9 => TimeMonth::September,
        10 => TimeMonth::October,
        11 => TimeMonth::November,
        12 => TimeMonth::December,
        _ => unreachable!(),
    };

    Date::from_calendar_date(date.year(), month, date.day() as u8)
        .expect("Invalid date conversion")
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
