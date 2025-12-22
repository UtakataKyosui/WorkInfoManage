use async_trait::async_trait;
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use std::path::PathBuf;
use crate::report::model::DailyReport;
use crate::report::storage::ReportStorage;

/// File-based storage for daily reports
pub struct FileReportStorage {
    base_dir: PathBuf,
}

impl FileReportStorage {
    /// Create a new file storage with default directory
    pub fn new() -> Result<Self> {
        // Use dirs crate for cross-platform home directory resolution
        let base_dir = if let Some(home) = dirs::home_dir() {
            home.join("task-manage").join("daily-report")
        } else {
            PathBuf::from("task-manage/daily-report")
        };
        
        // Create directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)
                .context("Failed to create daily-report directory")?;
        }
        
        Ok(Self { base_dir })
    }

    /// Create with custom directory (for testing)
    pub fn with_dir(base_dir: PathBuf) -> Result<Self> {
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)
                .context("Failed to create directory")?;
        }
        Ok(Self { base_dir })
    }

    /// Get file path for a specific date
    fn get_file_path(&self, date: NaiveDate) -> PathBuf {
        self.base_dir.join(format!("{}.md", date.format("%Y-%m-%d")))
    }
}

#[async_trait]
impl ReportStorage for FileReportStorage {
    async fn save_report(&self, report: &DailyReport) -> Result<()> {
        let path = self.get_file_path(report.date);
        
        // Save content to file
        tokio::fs::write(&path, &report.content).await
            .with_context(|| format!("Failed to write report to {}", path.display()))?;
        
        Ok(())
    }

    async fn load_report(&self, date: NaiveDate) -> Result<Option<DailyReport>> {
        let path = self.get_file_path(date);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = tokio::fs::read_to_string(&path).await
            .with_context(|| format!("Failed to read report from {}", path.display()))?;
        
        Ok(Some(DailyReport::new(date, content)))
    }

    async fn list_report_dates(&self, year: i32, month: u32) -> Result<Vec<NaiveDate>> {
        let mut dates = Vec::new();
        
        let mut entries = tokio::fs::read_dir(&self.base_dir).await
            .context("Failed to read directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if let Some(filename) = path.file_stem() {
                if let Some(filename_str) = filename.to_str() {
                    // Parse YYYY-MM-DD format
                    if let Ok(date) = NaiveDate::parse_from_str(filename_str, "%Y-%m-%d") {
                        if date.year() == year && date.month() == month {
                            dates.push(date);
                        }
                    }
                }
            }
        }
        
        dates.sort();
        Ok(dates)
    }

    async fn delete_report(&self, date: NaiveDate) -> Result<()> {
        let path = self.get_file_path(date);
        
        if path.exists() {
            tokio::fs::remove_file(&path).await
                .with_context(|| format!("Failed to delete report at {}", path.display()))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load_report() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileReportStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let report = DailyReport::new(date, "# Test Report\n\nContent here".to_string());
        
        storage.save_report(&report).await.unwrap();
        let loaded = storage.load_report(date).await.unwrap();
        
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().content, "# Test Report\n\nContent here");
    }

    #[tokio::test]
    async fn test_list_report_dates() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileReportStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let date1 = NaiveDate::from_ymd_opt(2025, 12, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 11, 16).unwrap();
        
        storage.save_report(&DailyReport::new(date1, "Day 1".to_string())).await.unwrap();
        storage.save_report(&DailyReport::new(date2, "Day 2".to_string())).await.unwrap();
        storage.save_report(&DailyReport::new(date3, "Day 3".to_string())).await.unwrap();
        
        let dates = storage.list_report_dates(2025, 12).await.unwrap();
        
        assert_eq!(dates.len(), 2);
        assert_eq!(dates[0], date1);
        assert_eq!(dates[1], date2);
    }

    #[tokio::test]
    async fn test_delete_report() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileReportStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let report = DailyReport::new(date, "Test".to_string());
        
        storage.save_report(&report).await.unwrap();
        assert!(storage.load_report(date).await.unwrap().is_some());
        
        storage.delete_report(date).await.unwrap();
        assert!(storage.load_report(date).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_file_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileReportStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
        
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let path = storage.get_file_path(date);
        
        assert!(path.to_string_lossy().ends_with("2025-12-16.md"));
    }
}
