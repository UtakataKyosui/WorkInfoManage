use work_info_manage::app::App;
use work_info_manage::ui::ui;
use work_info_manage::logic::sync::TaskSynchronizer;
use work_info_manage::config::Config;
use work_info_manage::storage::create_storage;

use anyhow::Context;
use std::{error::Error, io, sync::Arc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::{Backend, CrosstermBackend}, Terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    
    // Load configuration
    let config = Config::load()
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config.toml: {}", e);
            eprintln!("Using default JSON storage at ~/task-manage/data.json");
            
            Config {
                storage: work_info_manage::config::StorageConfig::json(),
            }
        });
    
    // Determine current storage type
    use work_info_manage::storage::{StorageState, StorageType, migrate_storage};
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
            eprintln!("\n⚠️  Storage type changed from {:?} to {:?}", prev_state.storage_type, current_type);
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
                    current_type.clone()
                ).await {
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
    let synchronizer = TaskSynchronizer::new();

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
    let report_storage: Arc<dyn work_info_manage::report::storage::ReportStorage> =
        Arc::new(work_info_manage::report::file_storage::FileReportStorage::new()
            .context("Failed to initialize report storage")?);

    // Create app state with storage
    let mut app = App::new(storage.clone(), report_storage);

    // Run app loop
    let res = run_app(&mut terminal, &mut app, &synchronizer, storage.clone()).await;

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

async fn save_current_memo(app: &mut App) {
    use work_info_manage::memo::Memo;

    let content = app.memo_state.memo_textarea.lines().join("\n");
    let memo = if let Some(path) = &app.memo_state.editing_memo_path {
        Memo {
            path: path.clone(),
            content,
            id: path.to_string_lossy().to_string(),
        }
    } else {
        Memo::new(content)
    };

    if let Ok(()) = memo.save() {
        app.memo_state.editing_memo_path = Some(memo.path);
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App, synchronizer: &TaskSynchronizer, storage: Arc<dyn work_info_manage::storage::Storage>) -> io::Result<()> {
    // Pomodoro timer configuration (15 minutes)
    const POMODORO_SECONDS: i64 = 15 * 60;
    
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if app.input_mode {
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
                } else {
                    // Global Keys
                     match key.code {
                        KeyCode::BackTab => {
                            // Shift+Tab to toggle calendar
                            if app.current_screen == work_info_manage::app::CurrentScreen::Calendar {
                                app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                                app.calendar_state = None;
                            } else {
                                app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                                app.calendar_state = Some(work_info_manage::app::CalendarState::new());
                            }
                        }
                        KeyCode::Char('t') => {
                            if app.timer.active_task_id.is_some() {
                                app.stop_timer().await;
                            } else {
                                app.start_timer();
                            }
                            // Continue processing - don't exit the app
                        }
                        _ => {}
                    }
                    
                    match app.current_screen {
                        work_info_manage::app::CurrentScreen::Menu => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.menu_selection > 0 {
                                        app.menu_selection -= 1;
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.menu_selection < 2 {
                                        app.menu_selection += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    match app.menu_selection {
                                        0 => {
                                            // Task Manager selected - load tasks if not already loaded
                                            if !app.tasks_loaded {
                                                app.status_message = "Loading tasks...".to_string();
                                                terminal.draw(|f| ui(f, app))?;

                                                app.load_tasks().await;

                                                // Auto-sync if configured
                                                app.status_message = "Syncing...".to_string();
                                                terminal.draw(|f| ui(f, app))?;

                                                match synchronizer.sync().await {
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

                                                        app.status_message = format!("Loaded {} tasks. Press 's' to refresh.", count);
                                                    }
                                                    Err(e) => {
                                                        app.sync_state.error = Some(e.to_string());
                                                        app.status_message = format!("Sync failed: {}. Press 's' to retry.", e);
                                                    }
                                                }

                                                app.tasks_loaded = true;
                                            }
                                            app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                                        }
                                        1 => {
                                            // Calendar & Daily Reports
                                            app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                                            app.calendar_state = Some(work_info_manage::app::CalendarState::new());
                                        }
                                        2 => {
                                            // Markdown Memos
                                            app.current_screen = work_info_manage::app::CurrentScreen::MemoList;
                                        }
                                        _ => {}
                                    }
                                }
                                KeyCode::Char('1') => {
                                    app.menu_selection = 0;
                                }
                                KeyCode::Char('2') => {
                                    app.menu_selection = 1;
                                }
                                KeyCode::Char('3') => {
                                    app.menu_selection = 2;
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::Dashboard => {
                            match key.code {
                                KeyCode::Char('q') => app.should_quit = true,
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Menu;
                                }
                                KeyCode::Char('m') => {
                                    // Go to Memo List
                                    app.current_screen = work_info_manage::app::CurrentScreen::MemoList;
                                }
                                KeyCode::Char('s') => {
                                    // Sync tasks
                                    app.status_message = "Syncing...".to_string();
                                    terminal.draw(|f| ui(f, app))?; // Redraw to show syncing message
                                    
                                    use std::io::Write;
                                    if let Err(e) = std::fs::create_dir_all("logs") {
                                        eprintln!("Warning: Failed to create logs directory: {}", e);
                                    }
                                    
                                    
                                    let mut log_file = match std::fs::OpenOptions::new()
                                        .create(true)
                                        .append(true)
                                        .open("logs/sync.log")
                                    {
                                        Ok(file) => Some(file),
                                        Err(e) => {
                                            eprintln!("Warning: Failed to open log file: {}", e);
                                            None
                                        }
                                    };
                                    
                                    if let Some(ref mut file) = log_file {
                                        writeln!(file, "\n=== Sync started at {:?} ===", std::time::SystemTime::now()).ok();
                                    }
                                    
                                    match synchronizer.sync().await {
                                        Ok(fetched_tasks) => {
                                            let count = fetched_tasks.len();
                                            if let Some(ref mut file) = log_file {
                                                writeln!(file, "Sync successful: {} tasks fetched", count).ok();
                                                for task in &fetched_tasks {
                                                    writeln!(file, "  - {}: {}", task.title, task.status).ok();
                                                }
                                            }
                                            app.tasks = fetched_tasks;
                                            app.ensure_selection_visible(); 
                                            
                                            // Update Sync State
                                            app.sync_state.last_sync_time = Some(chrono::Local::now());
                                            app.sync_state.total_tasks = count;
                                            app.sync_state.persisted_tasks = count;
                                            app.sync_state.error = None;
                                            
                                            app.status_message = format!("Sync completed. {} tasks loaded.", count);
                                        }
                                        Err(e) => {
                                            if let Some(ref mut file) = log_file {
                                                writeln!(file, "Sync error: {:?}", e).ok();
                                            }
                                            app.sync_state.error = Some(e.to_string());
                                            app.status_message = format!("Sync failed: {}", e);
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    app.select_next_task();
                                }
                                KeyCode::Up => {
                                    app.select_prev_task();
                                }
                                KeyCode::Enter => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Detail;
                                }
                                KeyCode::Char('T') => { // Shift+t
                                    app.current_screen = work_info_manage::app::CurrentScreen::Timer;
                                }
                                KeyCode::Char('r') => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::ReviewDetail;
                                }
                                KeyCode::Tab => {
                                    app.next_view();
                                }
                                KeyCode::Char('1') => {
                                    app.set_view(work_info_manage::app::CurrentView::Development);
                                }
                                KeyCode::Char('2') => {
                                    app.set_view(work_info_manage::app::CurrentView::InternalReview);
                                }
                                KeyCode::Char('3') => {
                                    app.set_view(work_info_manage::app::CurrentView::ExternalReview);
                                }
                                _ => {}
                            }
                        },
                        work_info_manage::app::CurrentScreen::Detail => {
                             match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                                }
                                KeyCode::Char('r') => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::ReviewDetail;
                                }
                                KeyCode::Char('n') => {
                                    app.input_mode = true;
                                    app.status_message = "Enter note (Enter to save, Esc to cancel)".to_string();
                                }
                                _ => {}
                            }
                        },
                        work_info_manage::app::CurrentScreen::Timer => {
                             match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                                }
                                _ => {}
                            }
                        },
                        work_info_manage::app::CurrentScreen::ReviewDetail => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Dashboard;
                                }
                                KeyCode::Enter => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Detail;
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::Calendar => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Menu;
                                    app.calendar_state = None;
                                }
                                KeyCode::Char('m') => {
                                    // Transition to Memo List
                                    app.current_screen = work_info_manage::app::CurrentScreen::MemoList;
                                }
                                KeyCode::Left => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Previous month
                                        state.current_month = state.current_month
                                            .checked_sub_months(chrono::Months::new(1))
                                            .unwrap_or(state.current_month);
                                    }
                                }
                                KeyCode::Right => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Next month
                                        state.current_month = state.current_month
                                            .checked_add_months(chrono::Months::new(1))
                                            .unwrap_or(state.current_month);
                                    }
                                }
                                KeyCode::Up => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Previous week (7 days)
                                        state.selected_date = state.selected_date
                                            .checked_sub_days(chrono::Days::new(7))
                                            .unwrap_or(state.selected_date);
                                    }
                                }
                                KeyCode::Down => {
                                    if let Some(ref mut state) = app.calendar_state {
                                        // Next week (7 days)
                                        state.selected_date = state.selected_date
                                            .checked_add_days(chrono::Days::new(7))
                                            .unwrap_or(state.selected_date);
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(ref state) = app.calendar_state {
                                        let selected_date = state.selected_date;

                                        // Load existing report if available
                                        let content = if let Ok(Some(report)) = app.report_storage.load_report(selected_date).await {
                                            report.content
                                        } else {
                                            String::new()
                                        };

                                        app.editor_state = Some(work_info_manage::app::EditorState::new(
                                            selected_date,
                                            content
                                        ));
                                        app.current_screen = work_info_manage::app::CurrentScreen::ReportEditor;
                                    }
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::ReportEditor => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                                    app.editor_state = None;
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::ReportPreview => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Calendar;
                                    app.preview_state = None;
                                }
                                KeyCode::Char('e') => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::ReportEditor;
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::MemoList => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::Menu;
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.memo_state.tree_state.key_down();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.memo_state.tree_state.key_up();
                                }
                                KeyCode::Left | KeyCode::Char('h') => {
                                    app.memo_state.tree_state.key_left();
                                }
                                KeyCode::Right | KeyCode::Char('l') => {
                                    app.memo_state.tree_state.key_right();
                                }
                                KeyCode::Char(' ') => {
                                    app.memo_state.tree_state.toggle_selected();
                                }
                                KeyCode::Char('n') => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::MemoEdit;
                                    app.memo_state.memo_textarea = tui_textarea::TextArea::default();
                                    work_info_manage::app::MemoState::configure_textarea(&mut app.memo_state.memo_textarea);
                                    app.memo_state.editing_memo_path = None;
                                }
                                KeyCode::Enter | KeyCode::Char('e') => {
                                    if let Some(selected_id) = app.memo_state.tree_state.selected().last() {
                                        let path_str = selected_id.split("::").next().unwrap_or("");

                                        if let Some(memo) = app.memo_state.memos.iter().find(|m| m.id == path_str) {
                                            app.current_screen = work_info_manage::app::CurrentScreen::MemoEdit;
                                            let lines: Vec<String> = memo.content.lines().map(|s| s.to_string()).collect();
                                            app.memo_state.memo_textarea = tui_textarea::TextArea::new(lines);

                                            if let Some(line_part) = selected_id.split("::").nth(1) {
                                                if let Ok(line_idx) = line_part.parse::<usize>() {
                                                    app.memo_state.memo_textarea.move_cursor(tui_textarea::CursorMove::Jump(line_idx as u16, 0));
                                                }
                                            }

                                            work_info_manage::app::MemoState::configure_textarea(&mut app.memo_state.memo_textarea);
                                            app.memo_state.editing_memo_path = Some(memo.path.clone());
                                        } else {
                                            app.memo_state.tree_state.toggle_selected();
                                        }
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if let Some(selected_id) = app.memo_state.tree_state.selected().last() {
                                        let path_str = selected_id.split("::").next().unwrap_or("");
                                        if let Some(memo) = app.memo_state.memos.iter().find(|m| m.id == path_str) {
                                            if memo.delete().is_ok() {
                                                let _ = app.memo_state.reload_memos();
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        work_info_manage::app::CurrentScreen::MemoEdit => {
                            use tui_textarea::Input;
                            use crossterm::event::KeyModifiers;

                            match key.code {
                                KeyCode::Esc => {
                                    // Save and return to list
                                    save_current_memo(&mut app).await;
                                    app.current_screen = work_info_manage::app::CurrentScreen::MemoList;
                                    let _ = app.memo_state.reload_memos();
                                }
                                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    save_current_memo(&mut app).await;
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.current_screen = work_info_manage::app::CurrentScreen::MemoList;
                                }
                                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if let Some(clipboard) = &mut app.memo_state.clipboard {
                                        if let Ok(text) = clipboard.get_text() {
                                            let text = text.replace("\r\n", "\n");
                                            app.memo_state.memo_textarea.insert_str(text);
                                        }
                                    }
                                }
                                _ => {
                                    app.memo_state.memo_textarea.input(Input::from(key));
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
                    eprintln!("⏰ Timer notification: 15 minutes passed! Cycle {}", app.timer.cycle_count);
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
