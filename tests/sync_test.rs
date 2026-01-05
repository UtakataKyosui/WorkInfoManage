use dotenvy;
use serde_json::json;
use std::fs::File;
use work_info_manage::logic::sync::TaskSynchronizer;

#[tokio::test]
async fn test_parse_asana_tasks() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let tasks_json = json!({
        "data": [
            {
                "gid": "12345",
                "name": "Test Task",
                "notes": "Test Notes",
                "resource_subtype": "default_task"
            },
            {
                "gid": "67890",
                "name": "Another Task",
                "notes": "More Notes",
                "resource_subtype": "default_task"
            }
        ]
    });

    let synchronizer = TaskSynchronizer::new();

    // Create a temp file for logging
    let mut log_file = File::create("test_sync.log").unwrap();

    let result = synchronizer.parse_asana_tasks(&tasks_json, &mut log_file);

    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 2);

    assert_eq!(tasks[0].asana_id, "12345");
    assert_eq!(tasks[0].title, "Test Task");
    assert_eq!(
        tasks[0].description,
        Some("[Type: default_task] Test Notes".to_string())
    );
    assert_eq!(tasks[0].status, "Imported");

    assert_eq!(tasks[1].asana_id, "67890");
    assert_eq!(tasks[1].title, "Another Task");

    // Cleanup
    std::fs::remove_file("test_sync.log").ok();
}
