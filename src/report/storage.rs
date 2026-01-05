use crate::report::model::DailyReport;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

/// Storage trait for daily reports
#[async_trait]
pub trait ReportStorage: Send + Sync {
    /// Save a daily report
    async fn save_report(&self, report: &DailyReport) -> Result<()>;

    /// Load a daily report for a specific date
    async fn load_report(&self, date: NaiveDate) -> Result<Option<DailyReport>>;

    /// List all dates that have reports in a given month
    async fn list_report_dates(&self, year: i32, month: u32) -> Result<Vec<NaiveDate>>;

    /// Delete a daily report
    async fn delete_report(&self, date: NaiveDate) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Mock storage for testing
    struct MockReportStorage {
        reports: Arc<RwLock<HashMap<NaiveDate, DailyReport>>>,
    }

    impl MockReportStorage {
        fn new() -> Self {
            Self {
                reports: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl ReportStorage for MockReportStorage {
        async fn save_report(&self, report: &DailyReport) -> Result<()> {
            let mut reports = self.reports.write().await;
            reports.insert(report.date, report.clone());
            Ok(())
        }

        async fn load_report(&self, date: NaiveDate) -> Result<Option<DailyReport>> {
            let reports = self.reports.read().await;
            Ok(reports.get(&date).cloned())
        }

        async fn list_report_dates(&self, year: i32, month: u32) -> Result<Vec<NaiveDate>> {
            let reports = self.reports.read().await;
            let mut dates: Vec<NaiveDate> = reports
                .keys()
                .filter(|date| date.year() == year && date.month() == month)
                .copied()
                .collect();
            dates.sort();
            Ok(dates)
        }

        async fn delete_report(&self, date: NaiveDate) -> Result<()> {
            let mut reports = self.reports.write().await;
            reports.remove(&date);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save_and_load_report() {
        let storage = MockReportStorage::new();
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let report = DailyReport::new(date, "Test content".to_string());

        storage.save_report(&report).await.unwrap();
        let loaded = storage.load_report(date).await.unwrap();

        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().content, "Test content");
    }

    #[tokio::test]
    async fn test_list_report_dates() {
        let storage = MockReportStorage::new();

        let date1 = NaiveDate::from_ymd_opt(2025, 12, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2025, 11, 16).unwrap();

        storage
            .save_report(&DailyReport::new(date1, "Day 1".to_string()))
            .await
            .unwrap();
        storage
            .save_report(&DailyReport::new(date2, "Day 2".to_string()))
            .await
            .unwrap();
        storage
            .save_report(&DailyReport::new(date3, "Day 3".to_string()))
            .await
            .unwrap();

        let dates = storage.list_report_dates(2025, 12).await.unwrap();

        assert_eq!(dates.len(), 2);
        assert_eq!(dates[0], date1);
        assert_eq!(dates[1], date2);
    }

    #[tokio::test]
    async fn test_delete_report() {
        let storage = MockReportStorage::new();
        let date = NaiveDate::from_ymd_opt(2025, 12, 16).unwrap();
        let report = DailyReport::new(date, "Test content".to_string());

        storage.save_report(&report).await.unwrap();
        assert!(storage.load_report(date).await.unwrap().is_some());

        storage.delete_report(date).await.unwrap();
        assert!(storage.load_report(date).await.unwrap().is_none());
    }
}
