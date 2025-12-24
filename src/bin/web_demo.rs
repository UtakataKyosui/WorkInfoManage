#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use web_time::Instant;
use ratzilla::DomBackend;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use work_info_manage::app::{App, CurrentScreen};
use work_info_manage::storage::Storage;
use work_info_manage::report::storage::ReportStorage;
use work_info_manage::db::{tasks, task_notes, work_logs};
use work_info_manage::report::model::DailyReport;
use work_info_manage::ui::ui;

struct WebStorage;
#[async_trait]
impl Storage for WebStorage {
    async fn load_tasks(&self) -> Result<Vec<tasks::Model>> {
        // Return dummy tasks for demo
        Ok(vec![
            tasks::Model {
                id: 1,
                asana_id: "DEMO-001".to_string(),
                title: "Implement WASM Build".to_string(),
                description: Some("Fix compilation errors and get trunk build working".to_string()),
                status: "In Progress".to_string(),
                priority: Some("high".to_string()),
                due_date: Some(chrono::Local::now().naive_local() + chrono::Duration::days(1)),
                github_pr_url: None,
                review_status: None,
                last_updated_at: chrono::Local::now().naive_local(),
            },
            tasks::Model {
                id: 2,
                asana_id: "DEMO-002".to_string(),
                title: "Add UI Animations".to_string(),
                description: Some("Implement smooth transitions and effects using tachyonfx".to_string()),
                status: "Not Started".to_string(),
                priority: Some("medium".to_string()),
                due_date: None,
                github_pr_url: None,
                review_status: None,
                last_updated_at: chrono::Local::now().naive_local() - chrono::Duration::days(1),
            },
            tasks::Model {
                id: 3,
                asana_id: "PR-42".to_string(),
                title: "Code Review: PR #42".to_string(),
                description: Some("Review authentication module changes".to_string()),
                status: "Internal Review UnChecked".to_string(),
                priority: Some("high".to_string()),
                due_date: Some(chrono::Local::now().naive_local()),
                github_pr_url: Some("https://github.com/example/repo/pull/42".to_string()),
                review_status: Some(serde_json::json!({"status": "pending", "reviewers": ["reviewer1"]})),
                last_updated_at: chrono::Local::now().naive_local(),
            },
        ])
    }
    async fn save_tasks(&self, _tasks: &[tasks::Model]) -> Result<()> { Ok(()) }
    async fn load_notes(&self, task_id: i32) -> Result<Vec<task_notes::Model>> {
        // Return dummy notes for demo
        if task_id == 1 {
            Ok(vec![
                task_notes::Model {
                    id: 1,
                    task_id: 1,
                    content: "Fixed crossterm dependency issue by using web-time".to_string(),
                    created_at: chrono::Local::now().naive_local() - chrono::Duration::hours(2),
                },
            ])
        } else {
            Ok(vec![])
        }
    }
    async fn load_all_notes(&self) -> Result<Vec<task_notes::Model>> { 
        Ok(vec![
            task_notes::Model {
                id: 1,
                task_id: 1,
                content: "Fixed crossterm dependency issue by using web-time".to_string(),
                created_at: chrono::Local::now().naive_local() - chrono::Duration::hours(2),
            },
        ])
    }
    async fn save_note(&self, note: &task_notes::Model) -> Result<task_notes::Model> { Ok(note.clone()) }
    async fn load_all_work_logs(&self) -> Result<Vec<work_logs::Model>> { Ok(vec![]) }
    async fn save_work_log(&self, _log: &work_logs::Model) -> Result<()> { Ok(()) }
}

struct WebReportStorage;
#[async_trait]
impl ReportStorage for WebReportStorage {
    async fn save_report(&self, _report: &DailyReport) -> Result<()> { Ok(()) }
    async fn load_report(&self, date: NaiveDate) -> Result<Option<DailyReport>> {
        // Return dummy report for today and yesterday
        let today = chrono::Local::now().naive_local().date();
        let yesterday = today - chrono::Days::new(1);
        
        if date == today {
            Ok(Some(DailyReport::new(
                date,
                "# Today's Progress\n\n## Completed\n- Fixed WASM build issues\n- Implemented cursor display\n\n## In Progress\n- Adding dummy data for demo\n\n## Notes\nWeb version is working well!".to_string()
            )))
        } else if date == yesterday {
            Ok(Some(DailyReport::new(
                date,
                "# Yesterday's Work\n\n## Completed\n- Set up ratzilla backend\n- Implemented basic navigation\n\n## Challenges\n- TextArea compatibility with WASM".to_string()
            )))
        } else {
            Ok(None)
        }
    }
    async fn list_report_dates(&self, year: i32, month: u32) -> Result<Vec<NaiveDate>> {
        use chrono::Datelike;
        let today = chrono::Local::now().naive_local().date();
        let yesterday = today - chrono::Days::new(1);
        
        // Return dates if they match the requested month
        let mut dates = Vec::new();
        if today.year() == year && today.month() == month {
            dates.push(today);
        }
        if yesterday.year() == year && yesterday.month() == month {
            dates.push(yesterday);
        }
        Ok(dates)
    }
    async fn delete_report(&self, _date: NaiveDate) -> Result<()> { Ok(()) }
}

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let backend = DomBackend::new()?;
    let mut terminal = ratatui::Terminal::new(backend)?;
    
    let storage = Arc::new(WebStorage);
    let report_storage = Arc::new(WebReportStorage);
    let app = Rc::new(RefCell::new(App::new(storage, report_storage)));
    
    // Set up keyboard event listener
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    
    let app_for_events = app.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut app = app_for_events.borrow_mut();
        
        // Convert web KeyboardEvent to our KeyEvent
        let key_code = match event.key().as_str() {
            "Enter" => work_info_manage::input::KeyCode::Enter,
            "Escape" => work_info_manage::input::KeyCode::Esc,
            "Backspace" => work_info_manage::input::KeyCode::Backspace,
            "ArrowLeft" => work_info_manage::input::KeyCode::Left,
            "ArrowRight" => work_info_manage::input::KeyCode::Right,
            "ArrowUp" => work_info_manage::input::KeyCode::Up,
            "ArrowDown" => work_info_manage::input::KeyCode::Down,
            "Home" => work_info_manage::input::KeyCode::Home,
            "End" => work_info_manage::input::KeyCode::End,
            "Tab" => work_info_manage::input::KeyCode::Tab,
            "Delete" => work_info_manage::input::KeyCode::Delete,
            s if s.len() == 1 => work_info_manage::input::KeyCode::Char(s.chars().next().unwrap()),
            _ => return,
        };
        
        let key_event = work_info_manage::input::KeyEvent {
            code: key_code,
            modifiers: work_info_manage::input::KeyModifiers {
                ctrl: event.ctrl_key(),
                alt: event.alt_key(),
                shift: event.shift_key(),
            },
        };
        
        // Handle input
        if app.input_mode {
            work_info_manage::input::InputHandler::handle_input_mode(&mut app, key_event);
        } else {
            match app.current_screen {
                CurrentScreen::Menu => {
                    work_info_manage::input::InputHandler::handle_menu(&mut app, key_event);
                    // Load tasks when entering Dashboard
                    if app.current_screen == CurrentScreen::Dashboard && !app.tasks_loaded {
                        app.tasks = work_info_manage::testing::create_dummy_tasks();
                        app.tasks_loaded = true;
                        app.ensure_selection_visible();
                    }
                }
                CurrentScreen::Dashboard => {
                    work_info_manage::input::InputHandler::handle_dashboard(&mut app, key_event);
                }
                CurrentScreen::Detail => {
                    work_info_manage::input::InputHandler::handle_detail(&mut app, key_event);
                }
                CurrentScreen::Calendar => {
                    work_info_manage::input::InputHandler::handle_calendar(&mut app, key_event);
                }
                CurrentScreen::ReportEditor => {}
                CurrentScreen::UnifiedMemoList => {
                    work_info_manage::input::InputHandler::handle_unified_memo_list(&mut app, key_event);
                }
                _ => {
                    if let work_info_manage::input::KeyCode::Esc = key_event.code {
                        app.current_screen = CurrentScreen::Menu;
                        app.animation.last_screen_change = Instant::now();
                    }
                }
            }
        }
        
        // Physics update is now handled in the render loop
    }) as Box<dyn FnMut(_)>);
    
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");
    
    document
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .expect("failed to add keydown listener");
    
    closure.forget();
    
    // Render loop using requestAnimationFrame
    let app_for_render = app.clone();
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Update physics
        {
            let mut app = app_for_render.borrow_mut();
            work_info_manage::logic::physics::update_physics(&mut app);
        }
        
        // Render
        let _ = terminal.draw(|frame| {
            let mut app = app_for_render.borrow_mut();
            ui(frame, &mut app);
        });
        
        // Schedule next frame
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    
    request_animation_frame(g.borrow().as_ref().unwrap());
    
    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("no global window")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("failed to request animation frame");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
