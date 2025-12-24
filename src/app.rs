use crate::storage::Storage;
use crate::memo::{Memo, load_memos};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Local, NaiveDate, Datelike};
use tui_textarea::TextArea;
use tui_tree_widget::TreeState;
#[cfg(not(target_arch = "wasm32"))]
use arboard::Clipboard;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct AnimationState {
    pub app_start_time: Instant,
    pub last_screen_change: Instant,
    pub visual_selection: f32, // Floating point index for smooth movement
    pub visual_velocity: f32,  // Velocity for spring physics
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            app_start_time: Instant::now(),
            last_screen_change: Instant::now(),
            visual_selection: 0.0,
            visual_velocity: 0.0,
        }
    }
}
pub struct TimerState {
    pub active_task_id: Option<i32>,
    pub start_time: Option<DateTime<Local>>,
    pub cycle_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentView {
    Development,
    InternalReview,
    ExternalReview,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentScreen {
    Menu,
    Dashboard,
    Detail,
    Timer,
    ReviewDetail,
    Calendar,
    ReportEditor,
    ReportPreview,
    UnifiedMemoList,
}

pub struct CalendarState {
    pub selected_date: NaiveDate,
    pub current_month: NaiveDate,
    pub report_dates: HashSet<NaiveDate>,
}

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}

impl CalendarState {
    pub fn new() -> Self {
        let today = chrono::Local::now().naive_local().date();
        Self {
            selected_date: today,
            current_month: today,
            report_dates: HashSet::new(),
        }
    }
}

pub struct EditorState {
    pub textarea: TextArea<'static>,
    pub date: NaiveDate,
}

impl EditorState {
    pub fn new(date: NaiveDate, content: String) -> Self {
        let mut textarea = TextArea::default();
        MemoState::configure_textarea(&mut textarea);
        textarea.insert_str(content);
        Self { textarea, date }
    }
}

pub struct PreviewState {
    pub date: NaiveDate,
    pub content: String,
    pub scroll_offset: usize,
}

impl PreviewState {
    pub fn new(date: NaiveDate, content: String) -> Self {
        Self {
            date,
            content,
            scroll_offset: 0,
        }
    }
}

pub struct SyncState {
    pub last_sync_time: Option<DateTime<Local>>,
    pub total_tasks: usize,
    pub persisted_tasks: usize,
    pub error: Option<String>,
}

pub struct MemoState {
    pub memos: Vec<Memo>,
    pub tree_state: TreeState<String>,
    pub memo_textarea: TextArea<'static>,
    pub editing_memo_path: Option<PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    pub clipboard: Option<Clipboard>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnifiedMemoItem {
    DailyReport { date: NaiveDate, title: String },
    TaskNote { task_id: i32, task_title: String, note_id: i32, created_at: chrono::NaiveDateTime },
}

impl UnifiedMemoItem {
    pub fn id(&self) -> String {
        match self {
            UnifiedMemoItem::DailyReport { date, .. } => format!("report-{}", date),
            UnifiedMemoItem::TaskNote { note_id, .. } => format!("note-{}", note_id),
        }
    }

    pub fn date(&self) -> NaiveDate {
        match self {
            UnifiedMemoItem::DailyReport { date, .. } => *date,
            UnifiedMemoItem::TaskNote { created_at, .. } => created_at.date(),
        }
    }
}

pub struct UnifiedMemoListState {
    pub items: Vec<UnifiedMemoItem>,
    pub tree_state: TreeState<String>,
}

impl Default for MemoState {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoState {
    pub fn new() -> Self {
        let memos = load_memos().unwrap_or_default();
        let mut tree_state = TreeState::default();
        let root_ids: Vec<String> = memos.iter().map(|m| m.id.clone()).collect();
        tree_state.open(root_ids);

        if !memos.is_empty() {
            tree_state.select_first();
        }

        let mut textarea = TextArea::default();
        Self::configure_textarea(&mut textarea);

        #[cfg(not(target_arch = "wasm32"))]
        let clipboard = Clipboard::new().ok();

        Self {
            memos,
            tree_state,
            memo_textarea: textarea,
            editing_memo_path: None,
            #[cfg(not(target_arch = "wasm32"))]
            clipboard,
        }
    }

    pub fn configure_textarea(textarea: &mut TextArea<'static>) {
        const SEARCH_PATTERN: &str = "(^#{1,6} .+$)|(\\*\\*.+?\\*\\*)";
        let _ = textarea.set_search_pattern(SEARCH_PATTERN);
        textarea.set_search_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD)
        );
    }

    pub fn reload_memos(&mut self) -> color_eyre::Result<()> {
        self.memos = load_memos()?;

        let mut new_state = TreeState::default();
        let root_ids: Vec<String> = self.memos.iter().map(|m| m.id.clone()).collect();
        new_state.open(root_ids);

        if !self.memos.is_empty() {
            new_state.select_first();
        }

        self.tree_state = new_state;
        Ok(())
    }
}

pub struct App {
    pub should_quit: bool,
    pub storage: Arc<dyn Storage>,
    pub report_storage: Arc<dyn crate::report::storage::ReportStorage>,
    pub tasks: Vec<crate::db::tasks::Model>,
    pub notes: HashMap<i32, Vec<crate::db::task_notes::Model>>,
    pub status_message: String,
    pub selected_task_index: usize,
    pub timer: TimerState,
    pub input_mode: bool,
    pub input_buffer: String,
    pub cursor_position: usize,
    pub task_note_textarea: Option<TextArea<'static>>,
    pub current_view: CurrentView,
    pub current_screen: CurrentScreen,
    pub sync_state: SyncState,
    pub calendar_state: Option<CalendarState>,
    pub editor_state: Option<EditorState>,
    pub preview_state: Option<PreviewState>,
    pub memo_state: MemoState,
    pub unified_memo_list_state: Option<UnifiedMemoListState>,
    pub menu_selection: usize,
    pub tasks_loaded: bool,
    pub sync_receiver: Option<std::sync::mpsc::Receiver<Result<Vec<crate::db::tasks::Model>, String>>>,
    pub animation: AnimationState,
} // App struct end

impl App {
    pub fn new(storage: Arc<dyn Storage>, report_storage: Arc<dyn crate::report::storage::ReportStorage>) -> Self {
        Self {
            should_quit: false,
            storage,
            report_storage,
            tasks: Vec::new(),
            notes: HashMap::new(),
            status_message: "Press 's' to sync, 't' to toggle timer, 'n' to add note, 'm' for memos, Shift+Tab for calendar, Esc for menu, q to quit.".to_string(),
            selected_task_index: 0,
            timer: TimerState {
                active_task_id: None,
                start_time: None,
                cycle_count: 0,
            },
            input_mode: false,
            input_buffer: String::new(),
            cursor_position: 0,
            task_note_textarea: None,
            current_view: CurrentView::Development,
            current_screen: CurrentScreen::Menu,
            sync_state: SyncState {
                last_sync_time: None,
                total_tasks: 0,
                persisted_tasks: 0,
                error: None,
            },
            calendar_state: None,
            editor_state: None,
            preview_state: None,
            memo_state: MemoState::new(),
            unified_memo_list_state: None,
            menu_selection: 0,
            tasks_loaded: false,
            sync_receiver: None,
            animation: AnimationState::default(),
        }
    }

    pub fn trigger_sync(&mut self, synchronizer: Arc<crate::logic::sync::TaskSynchronizer>) {
        if self.sync_receiver.is_some() {
            self.status_message = "Sync already in progress...".to_string();
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            let _ = synchronizer;
            self.status_message = "Sync not supported in Web Demo".to_string();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            self.sync_receiver = Some(rx);
            self.status_message = "Syncing in background...".to_string();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    synchronizer.sync().await
                });
                let _ = tx.send(result.map_err(|e| e.to_string()));
            });
        }
    }

    pub async fn load_tasks(&mut self) {
        // Load tasks from storage
        let tasks = self.storage.load_tasks().await.unwrap_or_default();
        self.tasks = tasks;

        // Load all notes in a single query to avoid N+1 problem
        self.notes.clear();
        if let Ok(all_notes) = self.storage.load_all_notes().await {
            // Group notes by task_id
            for note in all_notes {
                self.notes.entry(note.task_id).or_default().push(note);
            }
        }
    }

    pub fn start_timer(&mut self) {
        if !self.tasks.is_empty() {
            let task = &self.tasks[self.selected_task_index];
            self.timer.active_task_id = Some(task.id);
            self.timer.start_time = Some(Local::now());
            self.timer.cycle_count = 1;
            self.status_message = format!("Timer started for: {}", task.title);
        }
    }

    pub async fn stop_timer(&mut self) {
        if let (Some(start_time), Some(task_id)) = (self.timer.start_time, self.timer.active_task_id) {
            let end_time = Local::now();
            let duration = end_time.signed_duration_since(start_time).num_seconds();
            
            use crate::db::work_logs;
            use chrono::Utc;
            
            let work_log = work_logs::Model {
                id: 0,
                task_id,
                start_time: start_time.naive_local(),
                end_time: Some(end_time.naive_local()),
                duration_seconds: Some(duration as i32),
                created_at: Utc::now().naive_utc(),
            };
            
            if let Err(e) = self.storage.save_work_log(&work_log).await {
                eprintln!("Warning: Failed to save work log: {}", e);
                self.status_message = format!("Timer stopped. Duration: {}s (log save failed)", duration);
            } else {
                self.status_message = format!("Timer stopped. Duration: {}s", duration);
            }
        }
        
        self.timer.active_task_id = None;
        self.timer.start_time = None;
        self.timer.cycle_count = 0;
    }

    pub async fn save_note(&mut self) {
        let content = if let Some(ref textarea) = self.task_note_textarea {
            textarea.lines().join("\n")
        } else {
            self.input_buffer.clone()
        };

        if content.trim().is_empty() {
            return;
        }

        if let Some(task) = self.tasks.get(self.selected_task_index) {
            use crate::db::task_notes;
            use chrono::Utc;

            let note = task_notes::Model {
                id: 0,
                task_id: task.id,
                content,
                created_at: Utc::now().naive_utc(),
            };

            match self.storage.save_note(&note).await {
                Ok(saved_note) => {
                    self.notes.entry(task.id).or_default().push(saved_note);
                    self.status_message = "Note added.".to_string();
                }
                Err(e) => {
                    eprintln!("Failed to save note: {}", e);
                    self.status_message = "Failed to save note.".to_string();
                }
            }
        }
        self.input_buffer.clear();
        self.task_note_textarea = None;
        self.input_mode = false;
    }

    pub fn get_visible_statuses(&self) -> Vec<&'static str> {
        match self.current_view {
            CurrentView::Development => vec!["Not Started", "In Progress"],
            CurrentView::InternalReview => vec!["Internal Review UnChecked", "Internal Review Checked"],
            CurrentView::ExternalReview => vec!["External Review UnChecked", "External Review Checked"],
        }
    }

    pub fn is_task_visible(&self, task: &crate::db::tasks::Model) -> bool {
        let visible_statuses = self.get_visible_statuses();
        visible_statuses.contains(&task.status.as_str())
    }

    pub async fn build_unified_memo_list(&mut self) {
        let mut items = Vec::new();

        // Add Daily Reports from the last 12 months
        let now = chrono::Local::now().naive_local().date();
        for month_offset in 0..12 {
            let target_date = now - chrono::Months::new(month_offset);
            let year = target_date.year();
            let month = target_date.month();

            if let Ok(dates) = self.report_storage.list_report_dates(year, month).await {
                for date in dates {
                    items.push(UnifiedMemoItem::DailyReport {
                        date,
                        title: format!("📅 日次 {}", date.format("%Y-%m-%d")),
                    });
                }
            }
        }

        // Add Task Notes
        for (task_id, notes) in &self.notes {
            if let Some(task) = self.tasks.iter().find(|t| t.id == *task_id) {
                for note in notes {
                    items.push(UnifiedMemoItem::TaskNote {
                        task_id: *task_id,
                        task_title: task.title.clone(),
                        note_id: note.id,
                        created_at: note.created_at,
                    });
                }
            }
        }

        // Sort by date (newest first)
        items.sort_by(|a, b| {
            b.date().cmp(&a.date())
        });
        
        let mut tree_state = TreeState::default();
        tree_state.open(vec![format!("y-{}", now.year())]);

        self.unified_memo_list_state = Some(UnifiedMemoListState {
            items,
            tree_state,
        });
    }

    pub fn get_sorted_visible_indices(&self) -> Vec<usize> {
        let statuses = self.get_visible_statuses();
        let mut indices = Vec::new();
        
        for status in statuses {
            for (i, task) in self.tasks.iter().enumerate() {
                if task.status == status {
                    indices.push(i);
                }
            }
        }
        indices
    }

    pub fn select_next_task(&mut self) {
        if self.tasks.is_empty() { return; }
        
        let sorted_indices = self.get_sorted_visible_indices();
        if sorted_indices.is_empty() { return; }

        if let Some(pos) = sorted_indices.iter().position(|&i| i == self.selected_task_index) {
            if pos < sorted_indices.len() - 1 {
                self.selected_task_index = sorted_indices[pos + 1];
            }
        } else {
            // Selected task not visible or invalid, jump to first visible
            self.selected_task_index = sorted_indices[0];
        }
    }

    pub fn select_prev_task(&mut self) {
        if self.tasks.is_empty() { return; }
        
        let sorted_indices = self.get_sorted_visible_indices();
        if sorted_indices.is_empty() { return; }

        if let Some(pos) = sorted_indices.iter().position(|&i| i == self.selected_task_index) {
            if pos > 0 {
                self.selected_task_index = sorted_indices[pos - 1];
            }
        } else {
             // Selected task not visible or invalid, jump to first visible
            self.selected_task_index = sorted_indices[0];
        }
    }
    
    // Ensure selected task is always visible (e.g. after view switch)
    pub fn ensure_selection_visible(&mut self) {
        if self.tasks.is_empty() { return; }
        
        // If current selection is visible, do nothing
        if self.selected_task_index < self.tasks.len() && self.is_task_visible(&self.tasks[self.selected_task_index]) {
            return;
        }
        
        // Otherwise find first visible task
        for (idx, task) in self.tasks.iter().enumerate() {
            if self.is_task_visible(task) {
                self.selected_task_index = idx;
                return;
            }
        }
        
        // If no tasks are visible in this view, index doesn't matter much but let's reset to 0
        self.selected_task_index = 0;
    }

    pub fn next_view(&mut self) {
        self.current_view = match self.current_view {
            CurrentView::Development => CurrentView::InternalReview,
            CurrentView::InternalReview => CurrentView::ExternalReview,
            CurrentView::ExternalReview => CurrentView::Development,
        };
        self.ensure_selection_visible();
    }

    pub fn set_view(&mut self, view: CurrentView) {
        self.current_view = view;
        self.ensure_selection_visible();
    }
}
