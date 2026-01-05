// Platform-agnostic input handlers for each screen
use super::key_event::{KeyCode, KeyEvent};
use crate::app::{App, CurrentScreen};
use chrono::Datelike;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct InputHandler;

impl InputHandler {
    /// Handle input for the Menu screen
    pub fn handle_menu(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if app.menu_selection < 4 {
                    app.menu_selection += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.menu_selection > 0 {
                    app.menu_selection -= 1;
                }
            }
            KeyCode::Enter => {
                let prev = app.current_screen;
                match app.menu_selection {
                    0 => app.current_screen = CurrentScreen::Dashboard,
                    1 => {
                        app.current_screen = CurrentScreen::Calendar;
                        app.calendar_state = Some(crate::app::CalendarState::new());
                    }
                    2 => {
                        app.current_screen = CurrentScreen::UnifiedMemoList;
                        app.unified_memo_list_state = Some(crate::app::UnifiedMemoListState {
                            items: vec![],
                            tree_state: tui_tree_widget::TreeState::default(),
                        });
                    }
                    3 => {
                        app.current_screen = CurrentScreen::EnvManager;
                        if app.env_manager.is_none() {
                            if let Ok(mgr) = crate::logic::env_manager::EnvManager::new() {
                                app.env_manager = Some(mgr);
                            } else {
                                app.status_message = "Failed to load env manager".to_string();
                            }
                        }
                    }
                    4 => {
                        app.current_screen = CurrentScreen::ConfigManager;
                        if app.config_manager.is_none() {
                            if let Ok(mgr) = crate::logic::config_manager::ConfigManager::new() {
                                app.config_manager = Some(mgr);
                            } else {
                                app.status_message = "Failed to load config manager".to_string();
                            }
                        }
                    }
                    _ => {}
                }
                if app.current_screen != prev {
                    app.animation.last_screen_change = Instant::now();
                }
            }
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            _ => {}
        }
    }

    /// Handle input for the Dashboard screen
    pub fn handle_dashboard(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.current_screen = CurrentScreen::Menu;
                app.animation.last_screen_change = Instant::now();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.selected_task_index > 0 {
                    app.selected_task_index -= 1;
                    app.ensure_selection_visible();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.selected_task_index + 1 < app.tasks.len() {
                    app.selected_task_index += 1;
                    app.ensure_selection_visible();
                }
            }
            KeyCode::Enter => {
                if !app.tasks.is_empty() && app.selected_task_index < app.tasks.len() {
                    app.previous_screen = None; // Clear previous screen for normal navigation
                    app.current_screen = CurrentScreen::Detail;
                    app.animation.last_screen_change = Instant::now();
                }
            }
            KeyCode::Char('1') => app.set_view(crate::app::CurrentView::Development),
            KeyCode::Char('2') => app.set_view(crate::app::CurrentView::InternalReview),
            KeyCode::Char('3') => app.set_view(crate::app::CurrentView::ExternalReview),
            KeyCode::Tab | KeyCode::Char(' ') => app.next_view(),
            KeyCode::Char('q') => {
                app.should_quit = true;
            }
            _ => {}
        }
    }

    /// Handle input for the Detail screen
    pub fn handle_detail(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Return to previous screen if available, otherwise Dashboard
                let return_screen = app.previous_screen.unwrap_or(CurrentScreen::Dashboard);
                app.current_screen = return_screen;
                app.previous_screen = None;
                app.animation.last_screen_change = Instant::now();
            }
            KeyCode::Char('n') => {
                app.input_mode = true;
                app.input_buffer.clear();
                app.cursor_position = 0;
                app.status_message = "Adding note... (Esc to save)".to_string();
            }
            _ => {}
        }
    }

    /// Handle input for the Calendar screen
    pub fn handle_calendar(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.current_screen = CurrentScreen::Menu;
                app.calendar_state = None;
                app.animation.last_screen_change = Instant::now();
            }
            KeyCode::Char('[') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.current_month = state
                        .current_month
                        .checked_sub_months(chrono::Months::new(1))
                        .unwrap_or(state.current_month);
                }
            }
            KeyCode::Char(']') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.current_month = state
                        .current_month
                        .checked_add_months(chrono::Months::new(1))
                        .unwrap_or(state.current_month);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.selected_date = state
                        .selected_date
                        .checked_sub_days(chrono::Days::new(1))
                        .unwrap_or(state.selected_date);
                    let first_day = state.selected_date.with_day(1).unwrap();
                    if first_day != state.current_month {
                        state.current_month = first_day;
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.selected_date = state
                        .selected_date
                        .checked_add_days(chrono::Days::new(1))
                        .unwrap_or(state.selected_date);
                    let first_day = state.selected_date.with_day(1).unwrap();
                    if first_day != state.current_month {
                        state.current_month = first_day;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.selected_date = state
                        .selected_date
                        .checked_sub_days(chrono::Days::new(7))
                        .unwrap_or(state.selected_date);
                    let first_day = state.selected_date.with_day(1).unwrap();
                    if first_day != state.current_month {
                        state.current_month = first_day;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(ref mut state) = app.calendar_state {
                    state.selected_date = state
                        .selected_date
                        .checked_add_days(chrono::Days::new(7))
                        .unwrap_or(state.selected_date);
                    let first_day = state.selected_date.with_day(1).unwrap();
                    if first_day != state.current_month {
                        state.current_month = first_day;
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(ref state) = app.calendar_state {
                    let selected_date = state.selected_date;
                    app.input_mode = true;
                    app.editor_state =
                        Some(crate::app::EditorState::new(selected_date, String::new()));
                    app.current_screen = CurrentScreen::ReportEditor;
                }
            }
            _ => {}
        }
    }

    /// Handle input mode (text editing)
    pub fn handle_input_mode(app: &mut App, key: KeyEvent) {
        // Intercept Ctrl+s for Save
        if key.modifiers.ctrl && key.code == KeyCode::Char('s') {
            if app.current_screen == CurrentScreen::ConfigManager {
                if let Some(ref mut mgr) = app.config_manager {
                    mgr.update_database_url(app.input_buffer.clone());
                    mgr.save();
                    app.status_message = mgr.status_message.clone();
                }
                // Don't insert 's'
                return;
            }
        }

        match key.code {
            KeyCode::Esc => {
                app.input_mode = false;
                if app.editor_state.is_some() {
                    app.current_screen = CurrentScreen::Calendar;
                    app.editor_state = None;
                    app.status_message = "Report saved (mock).".to_string();
                } else if app.current_screen == CurrentScreen::EnvManager {
                    // Save edited value
                    if let Some(ref mut mgr) = app.env_manager {
                        if let Some(var) = mgr.variables.get_mut(mgr.selection) {
                            var.value = app.input_buffer.clone();
                        }
                    }
                    app.status_message = "Value updated. Press 's' to save to file.".to_string();
                } else if app.current_screen == CurrentScreen::ConfigManager {
                    // Save edited URL
                    if let Some(ref mut mgr) = app.config_manager {
                        mgr.update_database_url(app.input_buffer.clone());
                    }
                    app.status_message = "URL updated. Press 's' to save to file.".to_string();
                } else {
                    app.task_note_textarea = None;
                    app.status_message = "Note saved (mock).".to_string();
                }
                app.input_buffer.clear();
                app.cursor_position = 0;
            }
            KeyCode::Char(c) => {
                let pos = app.cursor_position;
                app.input_buffer.insert(pos, c);
                app.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    let pos = app.cursor_position;
                    app.input_buffer.remove(pos);
                }
            }
            KeyCode::Enter => {
                let pos = app.cursor_position;
                app.input_buffer.insert(pos, '\n');
                app.cursor_position += 1;
            }
            KeyCode::Left => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if app.cursor_position < app.input_buffer.len() {
                    app.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                let text_before = &app.input_buffer[..app.cursor_position];
                if let Some(last_newline) = text_before.rfind('\n') {
                    app.cursor_position = last_newline + 1;
                } else {
                    app.cursor_position = 0;
                }
            }
            KeyCode::End => {
                let text_after = &app.input_buffer[app.cursor_position..];
                if let Some(next_newline) = text_after.find('\n') {
                    app.cursor_position += next_newline;
                } else {
                    app.cursor_position = app.input_buffer.len();
                }
            }
            _ => {}
        }
    }

    /// Handle input for the UnifiedMemoList screen
    pub fn handle_unified_memo_list(app: &mut App, key: KeyEvent) {
        if let Some(ref mut state) = app.unified_memo_list_state {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.current_screen = CurrentScreen::Menu;
                    app.unified_memo_list_state = None;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state.tree_state.key_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.tree_state.key_up();
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    state.tree_state.key_left();
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    state.tree_state.key_right();
                }
                KeyCode::Char(' ') => {
                    state.tree_state.toggle_selected();
                }
                KeyCode::Enter => {
                    state.tree_state.toggle_selected();
                }
                _ => {}
            }
        }
    }

    /// Handle input for the ReviewDetail screen
    pub fn handle_review_detail(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.current_screen = CurrentScreen::Dashboard;
                app.animation.last_screen_change = Instant::now();
            }
            KeyCode::Enter => {
                app.current_screen = CurrentScreen::Detail;
                app.animation.last_screen_change = Instant::now();
            }
            _ => {}
        }
    }

    /// Handle input for the EnvManager screen
    pub fn handle_env_manager(app: &mut App, key: KeyEvent) {
        if let Some(ref mut mgr) = app.env_manager {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_screen = CurrentScreen::Menu;
                    // app.env_manager = None; // Keep state
                    app.animation.last_screen_change = Instant::now();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if mgr.selection + 1 < mgr.variables.len() {
                        mgr.selection += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if mgr.selection > 0 {
                        mgr.selection -= 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(var) = mgr.variables.get(mgr.selection) {
                        app.input_mode = true;
                        app.input_buffer = var.value.clone();
                        app.cursor_position = app.input_buffer.len();
                        app.status_message = format!("Editing {}", var.key);
                    }
                }
                KeyCode::Char('s') => {
                    if let Err(e) = mgr.save() {
                        app.status_message = format!("Failed to save: {}", e);
                    } else {
                        app.status_message = "Environment variables saved encrypted.".to_string();
                    }
                }
                _ => {}
            }
        }
    }
    /// Handle input for the ConfigManager screen
    pub fn handle_config_manager(app: &mut App, key: KeyEvent) {
        if let Some(ref mut mgr) = app.config_manager {
            // Handle Ctrl+s for save
            if key.modifiers.ctrl && key.code == KeyCode::Char('s') {
                mgr.save();
                app.status_message = mgr.status_message.clone();
                return;
            }

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_screen = CurrentScreen::Menu;
                    app.animation.last_screen_change = Instant::now();
                }
                KeyCode::Tab => {
                    if mgr.is_database() {
                        mgr.set_storage_type_json();
                    } else {
                        mgr.set_storage_type_database();
                    }
                }
                KeyCode::Char('1') | KeyCode::Left | KeyCode::Char('h') => {
                    mgr.set_storage_type_json();
                }
                KeyCode::Char('2') | KeyCode::Right | KeyCode::Char('l') => {
                    mgr.set_storage_type_database();
                }
                KeyCode::Enter => {
                    if mgr.is_database() {
                        app.input_mode = true;
                        app.input_buffer = mgr.get_database_url();
                        app.cursor_position = app.input_buffer.len();
                        app.status_message = "Editing Database URL".to_string();
                    }
                }
                _ => {}
            }
        }
    }
}
