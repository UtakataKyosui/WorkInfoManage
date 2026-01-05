use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".work"))
        .unwrap_or_else(|| PathBuf::from("."))
}
