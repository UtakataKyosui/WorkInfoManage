// Integration tests for input handling
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use std::sync::Arc;
use work_info_manage::app::{App, CurrentScreen, CurrentView};
use work_info_manage::db::{task_notes, tasks, work_logs};
use work_info_manage::input::{InputHandler, KeyCode, KeyEvent};
use work_info_manage::report::storage::ReportStorage;
use work_info_manage::report::DailyReport;
use work_info_manage::storage::Storage;

struct MockStorage;

#[async_trait]
impl Storage for MockStorage {
    async fn load_tasks(&self) -> Result<Vec<tasks::Model>> {
        Ok(vec![])
    }
    async fn save_tasks(&self, _: &[tasks::Model]) -> Result<()> {
        Ok(())
    }
    async fn load_notes(&self, _: i32) -> Result<Vec<task_notes::Model>> {
        Ok(vec![])
    }
    async fn load_all_notes(&self) -> Result<Vec<task_notes::Model>> {
        Ok(vec![])
    }
    async fn save_note(&self, note: &task_notes::Model) -> Result<task_notes::Model> {
        Ok(note.clone())
    }
    async fn load_all_work_logs(&self) -> Result<Vec<work_logs::Model>> {
        Ok(vec![])
    }
    async fn save_work_log(&self, log: &work_logs::Model) -> Result<()> {
        Ok(())
    }
}

struct MockReportStorage;

#[async_trait]
impl ReportStorage for MockReportStorage {
    async fn load_report(&self, _: NaiveDate) -> Result<Option<DailyReport>> {
        Ok(None)
    }
    async fn save_report(&self, _: &DailyReport) -> Result<()> {
        Ok(())
    }
    async fn delete_report(&self, _: NaiveDate) -> Result<()> {
        Ok(())
    }
    async fn list_report_dates(&self, _: i32, _: u32) -> Result<Vec<NaiveDate>> {
        Ok(vec![])
    }
}

fn create_test_app() -> App {
    App::new(Arc::new(MockStorage), Arc::new(MockReportStorage))
}

#[test]
fn test_menu_navigation() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Menu;
    app.menu_selection = 0;

    InputHandler::handle_menu(&mut app, KeyEvent::new(KeyCode::Char('j')));
    assert_eq!(app.menu_selection, 1);

    InputHandler::handle_menu(&mut app, KeyEvent::new(KeyCode::Char('k')));
    assert_eq!(app.menu_selection, 0);
}

#[test]
fn test_menu_selection() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Menu;
    app.menu_selection = 0;

    InputHandler::handle_menu(&mut app, KeyEvent::new(KeyCode::Enter));
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);
}

#[test]
fn test_dashboard_navigation() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Dashboard;
    app.tasks = work_info_manage::testing::create_dummy_tasks();
    app.selected_task_index = 0;

    InputHandler::handle_dashboard(&mut app, KeyEvent::new(KeyCode::Char('j')));
    assert_eq!(app.selected_task_index, 1);

    InputHandler::handle_dashboard(&mut app, KeyEvent::new(KeyCode::Char('k')));
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_dashboard_view_switching() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Dashboard;
    app.current_view = CurrentView::Development;

    InputHandler::handle_dashboard(&mut app, KeyEvent::new(KeyCode::Char('2')));
    assert_eq!(app.current_view, CurrentView::InternalReview);
}

#[test]
fn test_detail_navigation() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Detail;

    InputHandler::handle_detail(&mut app, KeyEvent::new(KeyCode::Esc));
    assert_eq!(app.current_screen, CurrentScreen::Dashboard);
}

#[test]
fn test_calendar_navigation() {
    let mut app = create_test_app();
    app.current_screen = CurrentScreen::Calendar;
    app.calendar_state = Some(work_info_manage::app::CalendarState::new());

    let initial_date = app.calendar_state.as_ref().unwrap().selected_date;
    InputHandler::handle_calendar(&mut app, KeyEvent::new(KeyCode::Right));
    assert!(app.calendar_state.as_ref().unwrap().selected_date > initial_date);
}
