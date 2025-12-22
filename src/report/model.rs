use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Daily report data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyReport {
    /// Date of the report (YYYY-MM-DD)
    pub date: NaiveDate,
    /// Markdown content of the report
    pub content: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl DailyReport {
    /// Create a new daily report
    pub fn new(date: NaiveDate, content: String) -> Self {
        let now = Utc::now();
        Self {
            date,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the content and timestamp
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_new_daily_report() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let content = "# Daily Report\n\nTest content".to_string();
        
        let report = DailyReport::new(date, content.clone());
        
        assert_eq!(report.date, date);
        assert_eq!(report.content, content);
        assert!(report.created_at <= Utc::now());
        assert_eq!(report.created_at, report.updated_at);
    }

    #[test]
    fn test_update_content() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let mut report = DailyReport::new(date, "Initial content".to_string());
        
        let original_created = report.created_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        report.update_content("Updated content".to_string());
        
        assert_eq!(report.content, "Updated content");
        assert_eq!(report.created_at, original_created);
        assert!(report.updated_at > report.created_at);
    }
}
