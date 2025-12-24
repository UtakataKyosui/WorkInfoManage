pub mod model;
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod file_storage;

pub use model::DailyReport;
pub use storage::ReportStorage;
#[cfg(not(target_arch = "wasm32"))]
pub use file_storage::FileReportStorage;
