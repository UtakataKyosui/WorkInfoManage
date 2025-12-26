use work_info_manage::app::App;
use work_info_manage::config::Config;
use work_info_manage::logic::sync::TaskSynchronizer;
use work_info_manage::storage::create_storage;
use work_info_manage::ui::ui;
mod markdown_utils;

use anyhow::Context;
use chrono::Datelike;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, sync::Arc};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config.toml: {}", e);
        eprintln!("Using default JSON storage at ~/task-manage/data.json");

        Config {
            storage: work_info_manage::config::StorageConfig::json(),
        }
    });

    // Determine current storage type
    use work_info_manage::storage::{migrate_storage, StorageState, StorageType};
    let current_type = match &config.storage {
        work_info_manage::config::StorageConfig::Json { .. } => StorageType::Json,
        work_info_manage::config::StorageConfig::Database { .. } => StorageType::Database,
    };

    // Check if storage type changed
    let previous_state = StorageState::load();
    let needs_migration = if let Some(prev) = &previous_state {
        prev.storage_type != current_type
    } else {
        false
    };

    // Create storage backend from config
    let storage = create_storage(&config).await?;

    // Perform migration if needed
    if needs_migration {
        if let Some(prev_state) = previous_state {
            eprintln!(
                "\n⚠️  Storage type changed from {:?} to {:?}",
                prev_state.storage_type, current_type
            );
            eprintln!("Starting automatic data migration...\n");

            // Create previous storage using saved config
            let prev_config = Config {
                storage: prev_state.storage_config.clone(),
            };

            if let Ok(prev_storage) = create_storage(&prev_config).await {
                if let Err(e) = migrate_storage(
                    prev_storage,
                    storage.clone(),
                    prev_state.storage_type.clone(),
                    current_type.clone(),
                )
                .await
                {
                    eprintln!("❌ Migration failed: {}", e);
                    eprintln!("You may need to manually migrate your data.");
                }
            } else {
                eprintln!("❌ Could not connect to previous storage backend. Migration skipped.");
                eprintln!("Your new storage will start empty. You may need to manually migrate your data or sync from Asana/GitHub.");
            }
        }
    }

    // Save current storage state
    let new_state = StorageState::new(current_type, config.storage.clone());
    if let Err(e) = new_state.save() {
        eprintln!("Warning: Failed to save storage state: {}", e);
    }

    // Setup Synchronizer
    let synchronizer = Arc::new(TaskSynchronizer::new());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Create report storage
    let report_storage: Arc<dyn work_info_manage::report::storage::ReportStorage> = Arc::new(
        work_info_manage::report::file_storage::FileReportStorage::new()
            .context("Failed to initialize report storage")?,
    );

    // Create app state with storage
    let mut app = App::new(storage.clone(), report_storage);

    // Run app loop
    let res = run_app(
        &mut terminal,
        &mut app,
        synchronizer.clone(),
        storage.clone(),
    )
    .await;

    // Cleanup terminal - ensure this always happens
    // Clear the terminal before leaving alternate screen
    terminal.clear()?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print error if any
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    synchronizer: Arc<TaskSynchronizer>,
    storage: Arc<dyn work_info_manage::storage::Storage>,
) -> io::Result<()> {
    // Pomodoro timer configuration (15 minutes)
    const POMODORO_SECONDS: i64 = 15 * 60;

    let mut last_tick = std::time::Instant::now();

    loop {
        // Calculate delta time for animations
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f32();
        last_tick = now;

        app.tick(dt);

        terminal.draw(|f| ui(f, app))?;

        // Check for sync results
        if let Some(ref rx) = app.sync_receiver {
            if let Ok(result) = rx.try_recv() {
                // Sync finished
                app.sync_receiver = None;

                match result {
                    Ok(fetched_tasks) => {
                        let count = fetched_tasks.len();
                        if let Err(e) = storage.save_tasks(&fetched_tasks).await {
                            eprintln!("Warning: Failed to save synced tasks: {}", e);
                        }
                        app.tasks = fetched_tasks;
                        app.ensure_selection_visible();

                        app.sync_state.last_sync_time = Some(chrono::Local::now());
                        app.sync_state.total_tasks = count;
                        app.sync_state.persisted_tasks = count;
                        app.sync_state.error = None;

                        app.status_message = format!("Sync completed. {} tasks loaded.", count);
                    }
                    Err(e) => {
                        app.sync_state.error = Some(e.clone());
                        app.status_message = format!("Sync failed: {}. Press 's' to retry.", e);
                    }
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global Cycle Navigation (Shift+Tab)
                if key.code == KeyCode::BackTab {
                    app.input_mode = false;
                    app.task_note_textarea = None;

                    match app.current_screen {
                        work_info_manage::app::CurrentScreen::Menu => {
                            app.build_unified_memo_list().await;
                            app.current_screen =
                                work_info_manage::app::CurrentScreen::UnifiedMemoList;
                        }
                        work_info_manage::app::CurrentScreen::UnifiedMemoList => {
                            if app.calendar_state.is_none() {
                                app.calendar_state =
                                    Some(work_info_manage::app::CalendarState::new());
                            }
                            app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                        }
                        work_info_manage::app::CurrentScreen::Calendar
                        | work_info_manage::app::CurrentScreen::ReportEditor
                        | work_info_manage::app::CurrentScreen::ReportPreview => {
                            if !app.tasks_loaded {
                                app.load_tasks().await;
                                app.trigger_sync(synchronizer.clone());
                                app.tasks_loaded = true;
                            }
                            app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                        }
                        _ => {
                            // Dashboard, Detail, Timer, ReviewDetail -> Menu
                            app.current_screen = work_info_manage::app::CurrentScreen::Menu;
                        }
                    }
                    continue;
                }

                if app.input_mode {
                    if let Some(ref mut textarea) = app.task_note_textarea {
                        // TextArea mode - Markdown editing
                        match key.code {
                            KeyCode::Enter => {
                                crate::markdown_utils::handle_markdown_enter(textarea);
                            }
                            KeyCode::Esc => {
                                app.save_note().await;
                            }
                            KeyCode::Char('c')
                                if key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
                            {
                                app.input_mode = false;
                                app.task_note_textarea = None;
                                app.status_message = "Note cancelled.".to_string();
                            }
                            KeyCode::Char('v')
                                if key
                                    .modifiers
                                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
                            {
                                if let Some(ref mut clipboard) = app.memo_state.clipboard {
                                    if let Ok(text) = clipboard.get_text() {
                                        textarea.insert_str(text);
                                    }
                                }
                            }
                            KeyCode::Char('c')
                                if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) =>
                            {
                                if let Some(ref mut clipboard) = app.memo_state.clipboard {
                                    let text = textarea.lines().join("\n");
                                    let _ = clipboard.set_text(text);
                                }
                            }
                            _ => {
                                textarea.input(key);
                            }
                        }
                    } else {
                        // Fallback to simple input buffer
                        match key.code {
                            KeyCode::Enter => {
                                app.save_note().await;
                            }
                            KeyCode::Esc => {
                                app.input_mode = false;
                                app.input_buffer.clear();
                                app.status_message = "Note cancelled.".to_string();
                            }
                            KeyCode::Char(c) => {
                                app.input_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Global Keys
                    if let KeyCode::Char('t') = key.code {
                        if app.timer.active_task_id.is_some() {
                            app.stop_timer().await;
                        } else {
                            app.start_timer();
                        }
                        // Continue processing - don't exit the app
                    }

                    match app.current_screen {
                        work_info_manage::app::CurrentScreen::Menu => {
                            // Convert crossterm KeyEvent to platform-agnostic KeyEvent
                            let key_event = work_info_manage::input::KeyEvent::from(key);

                            // Use shared input handler
                            work_info_manage::input::InputHandler::handle_menu(
                                &mut *app, key_event,
                            );

                            // Handle screen-specific initialization after transition
                            match app.current_screen {
                                work_info_manage::app::CurrentScreen::Dashboard => {
                                    if !app.tasks_loaded {
                                        app.load_tasks().await;
                                        app.trigger_sync(synchronizer.clone());
                                        app.tasks_loaded = true;
                                    }
                                }
                                work_info_manage::app::CurrentScreen::Calendar => {
                                    app.calendar_state =
                                        Some(work_info_manage::app::CalendarState::new());
                                }
                                work_info_manage::app::CurrentScreen::UnifiedMemoList => {
                                    app.build_unified_memo_list().await;
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::Dashboard => {
                            let key_event = work_info_manage::input::KeyEvent::from(key);

                            // Handle special sync action before general handling
                            let is_sync = matches!(
                                key_event.code,
                                work_info_manage::input::KeyCode::Char('s')
                            );

                            work_info_manage::input::InputHandler::handle_dashboard(
                                &mut *app, key_event,
                            );

                            if is_sync {
                                app.trigger_sync(synchronizer.clone());
                            }
                        }
                        work_info_manage::app::CurrentScreen::Detail => {
                            let key_event = work_info_manage::input::KeyEvent::from(key);
                            work_info_manage::input::InputHandler::handle_detail(
                                &mut *app, key_event,
                            );
                        }
                        work_info_manage::app::CurrentScreen::Timer => {
                            if key.code == KeyCode::Esc {
                                app.current_screen =
                                    work_info_manage::app::CurrentScreen::Dashboard;
                            }
                        }
                        work_info_manage::app::CurrentScreen::ReviewDetail => {
                            let key_event = work_info_manage::input::KeyEvent::from(key);
                            work_info_manage::input::InputHandler::handle_review_detail(
                                &mut *app, key_event,
                            );
                        }
                        work_info_manage::app::CurrentScreen::Calendar => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Menu;
                                    app.calendar_state = None;
                                }
                                KeyCode::Char('[') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Previous month
                                        state.current_month = state
                                            .current_month
                                            .checked_sub_months(chrono::Months::new(1))
                                            .unwrap_or(state.current_month);
                                    }
                                }
                                KeyCode::Char(']') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Next month
                                        state.current_month = state
                                            .current_month
                                            .checked_add_months(chrono::Months::new(1))
                                            .unwrap_or(state.current_month);
                                    }
                                }
                                KeyCode::Left | KeyCode::Char('h') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Previous day
                                        state.selected_date = state
                                            .selected_date
                                            .checked_sub_days(chrono::Days::new(1))
                                            .unwrap_or(state.selected_date);
                                        // Sync view if moved to prev month
                                        let first_day = state.selected_date.with_day(1).unwrap();
                                        if first_day != state.current_month {
                                            state.current_month = first_day;
                                        }
                                    }
                                }
                                KeyCode::Right | KeyCode::Char('l') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Next day
                                        state.selected_date = state
                                            .selected_date
                                            .checked_add_days(chrono::Days::new(1))
                                            .unwrap_or(state.selected_date);
                                        // Sync view if moved to next month
                                        let first_day = state.selected_date.with_day(1).unwrap();
                                        if first_day != state.current_month {
                                            state.current_month = first_day;
                                        }
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Previous week
                                        state.selected_date = state
                                            .selected_date
                                            .checked_sub_days(chrono::Days::new(7))
                                            .unwrap_or(state.selected_date);
                                        // Sync view
                                        let first_day = state.selected_date.with_day(1).unwrap();
                                        if first_day != state.current_month {
                                            state.current_month = first_day;
                                        }
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Next week
                                        state.selected_date = state
                                            .selected_date
                                            .checked_add_days(chrono::Days::new(7))
                                            .unwrap_or(state.selected_date);
                                        // Sync view
                                        let first_day = state.selected_date.with_day(1).unwrap();
                                        if first_day != state.current_month {
                                            state.current_month = first_day;
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(ref state) = app.calendar_state {
                                        let selected_date = state.selected_date;

                                        // Load existing report if available
                                        let content = if let Ok(Some(report)) =
                                            app.report_storage.load_report(selected_date).await
                                        {
                                            report.content
                                        } else {
                                            String::new()
                                        };

                                        app.editor_state =
                                            Some(work_info_manage::app::EditorState::new(
                                                selected_date,
                                                content,
                                            ));
                                        app.previous_screen = Some(app.current_screen);
                                        app.current_screen =
                                            work_info_manage::app::CurrentScreen::ReportEditor;
                                    }
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::ReportEditor => {
                            if let Some(ref mut editor) = app.editor_state {
                                match key.code {
                                    KeyCode::Enter => {
                                        crate::markdown_utils::handle_markdown_enter(
                                            &mut editor.textarea,
                                        );
                                    }
                                    KeyCode::Esc => {
                                        // Save and Exit
                                        let content = editor.textarea.lines().join("\n");
                                        let report = work_info_manage::report::DailyReport::new(
                                            editor.date,
                                            content,
                                        );
                                        if let Err(e) =
                                            app.report_storage.save_report(&report).await
                                        {
                                            app.status_message =
                                                format!("Failed to save report: {}", e);
                                        } else {
                                            app.status_message = "Report saved.".to_string();

                                            // Return to previous screen
                                            let return_screen = app.previous_screen.unwrap_or(
                                                work_info_manage::app::CurrentScreen::Calendar,
                                            );

                                            // Ensure required state exists for the return screen
                                            match return_screen {
                                                work_info_manage::app::CurrentScreen::Calendar => {
                                                    if app.calendar_state.is_none() {
                                                        app.calendar_state = Some(work_info_manage::app::CalendarState::new());
                                                    }
                                                }
                                                work_info_manage::app::CurrentScreen::UnifiedMemoList => {
                                                    // Rebuild the memo list when returning
                                                    app.build_unified_memo_list().await;
                                                }
                                                _ => {}
                                            }

                                            app.current_screen = return_screen;
                                            app.previous_screen = None;
                                            app.editor_state = None;
                                        }
                                    }
                                    KeyCode::Char('s')
                                        if key
                                            .modifiers
                                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                                    {
                                        // Save only
                                        let content = editor.textarea.lines().join("\n");
                                        let report = work_info_manage::report::DailyReport::new(
                                            editor.date,
                                            content,
                                        );
                                        if let Err(e) =
                                            app.report_storage.save_report(&report).await
                                        {
                                            app.status_message =
                                                format!("Failed to save report: {}", e);
                                        } else {
                                            app.status_message = "Report saved.".to_string();
                                        }
                                    }
                                    KeyCode::Char('c')
                                        if key
                                            .modifiers
                                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                                    {
                                        // Cancel without saving
                                        let return_screen = app.previous_screen.unwrap_or(
                                            work_info_manage::app::CurrentScreen::Calendar,
                                        );

                                        // Ensure required state exists for the return screen
                                        match return_screen {
                                            work_info_manage::app::CurrentScreen::Calendar => {
                                                if app.calendar_state.is_none() {
                                                    app.calendar_state = Some(work_info_manage::app::CalendarState::new());
                                                }
                                            }
                                            work_info_manage::app::CurrentScreen::UnifiedMemoList => {
                                                // Rebuild the memo list when returning
                                                app.build_unified_memo_list().await;
                                            }
                                            _ => {}
                                        }

                                        app.current_screen = return_screen;
                                        app.previous_screen = None;
                                        app.editor_state = None;
                                        app.status_message = "Cancelled.".to_string();
                                    }
                                    KeyCode::Char('v')
                                        if key
                                            .modifiers
                                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                                    {
                                        if let Some(ref mut clipboard) = app.memo_state.clipboard {
                                            if let Ok(text) = clipboard.get_text() {
                                                editor.textarea.insert_str(text);
                                            }
                                        }
                                    }
                                    KeyCode::Char('c')
                                        if key
                                            .modifiers
                                            .contains(crossterm::event::KeyModifiers::ALT) =>
                                    {
                                        if let Some(ref mut clipboard) = app.memo_state.clipboard {
                                            let text = editor.textarea.lines().join("\n");
                                            let _ = clipboard.set_text(text);
                                        }
                                    }
                                    _ => {
                                        editor.textarea.input(key);
                                    }
                                }
                            }
                        }
                        work_info_manage::app::CurrentScreen::ReportPreview => match key.code {
                            KeyCode::Esc => {
                                app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                                app.preview_state = None;
                            }
                            KeyCode::Char('e') => {
                                app.current_screen =
                                    work_info_manage::app::CurrentScreen::ReportEditor;
                            }
                            _ => {}
                        },
                        work_info_manage::app::CurrentScreen::UnifiedMemoList => {
                            if let Some(ref mut state) = app.unified_memo_list_state {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => {
                                        app.current_screen =
                                            work_info_manage::app::CurrentScreen::Menu;
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
                                        let selected = state.tree_state.selected();
                                        if !selected.is_empty() {
                                            let last_segment = selected.last().unwrap();

                                            // Check ID type
                                            if last_segment.starts_with("report-") {
                                                // It's a daily report
                                                // Format: report-YYYY-MM-DD
                                                if let Ok(date) = chrono::NaiveDate::parse_from_str(
                                                    last_segment.trim_start_matches("report-"),
                                                    "%Y-%m-%d",
                                                ) {
                                                    let content = if let Ok(Some(report)) =
                                                        app.report_storage.load_report(date).await
                                                    {
                                                        report.content
                                                    } else {
                                                        String::new()
                                                    };

                                                    app.editor_state = Some(
                                                        work_info_manage::app::EditorState::new(
                                                            date, content,
                                                        ),
                                                    );
                                                    app.previous_screen = Some(app.current_screen);
                                                    app.current_screen = work_info_manage::app::CurrentScreen::ReportEditor;
                                                }
                                            } else if last_segment.starts_with("note-") {
                                                // It's a task note
                                                // Find the note to jump to task
                                                if let Ok(note_id) = last_segment
                                                    .trim_start_matches("note-")
                                                    .parse::<i32>()
                                                {
                                                    // Find which task has this note
                                                    let mut target_task_index = None;
                                                    for (idx, task) in app.tasks.iter().enumerate()
                                                    {
                                                        if let Some(notes) = app.notes.get(&task.id)
                                                        {
                                                            if notes.iter().any(|n| n.id == note_id)
                                                            {
                                                                target_task_index = Some(idx);
                                                                break;
                                                            }
                                                        }
                                                    }

                                                    if let Some(idx) = target_task_index {
                                                        app.selected_task_index = idx;
                                                        app.previous_screen =
                                                            Some(app.current_screen);
                                                        app.current_screen = work_info_manage::app::CurrentScreen::Detail;
                                                    }
                                                }
                                            } else {
                                                // Group node (Year/Month/Day) - Toggle
                                                state.tree_state.toggle_selected();
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check timer
        if let Some(start_time) = app.timer.start_time {
            let now = chrono::Local::now();
            let elapsed = now.signed_duration_since(start_time).num_seconds();

            // Check if Pomodoro timer has elapsed
            if elapsed >= POMODORO_SECONDS {
                #[cfg(target_os = "macos")]
                {
                    // Play sound (macOS only)
                    if let Err(e) = std::process::Command::new("afplay")
                        .arg("/System/Library/Sounds/Glass.aiff")
                        .spawn()
                    {
                        eprintln!("Warning: Failed to play notification sound: {}", e);
                    }

                    // Show notification (macOS only)
                    if let Err(e) = std::process::Command::new("osascript")
                        .arg("-e")
                        .arg(format!("display notification \"15 minutes passed! Cycle {}\" with title \"TaskManager Timer\"", app.timer.cycle_count))
                        .spawn()
                    {
                        eprintln!("Warning: Failed to show notification: {}", e);
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    // Fallback for non-macOS platforms
                    eprintln!(
                        "⏰ Timer notification: 15 minutes passed! Cycle {}",
                        app.timer.cycle_count
                    );
                }

                app.timer.cycle_count += 1;
                app.timer.start_time = Some(now);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
