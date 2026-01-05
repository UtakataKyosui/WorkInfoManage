// Dummy data generation for testing and WASM builds
use crate::db::tasks;
use chrono::Local;

/// Create dummy tasks for demo/testing purposes
pub fn create_dummy_tasks() -> Vec<tasks::Model> {
    vec![
        // Development - Not Started
        tasks::Model {
            id: 1,
            asana_id: "DEMO-001".to_string(),
            title: "Setup CI/CD Pipeline".to_string(),
            description: Some(
                "Configure GitHub Actions for automated testing and deployment".to_string(),
            ),
            status: "Not Started".to_string(),
            priority: Some("high".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(3)),
            github_pr_url: None,
            review_status: None,
            last_updated_at: Local::now().naive_local(),
        },
        tasks::Model {
            id: 2,
            asana_id: "DEMO-002".to_string(),
            title: "Add Database Migration Tool".to_string(),
            description: Some("Implement schema versioning and migration scripts".to_string()),
            status: "Not Started".to_string(),
            priority: Some("medium".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(7)),
            github_pr_url: None,
            review_status: None,
            last_updated_at: Local::now().naive_local() - chrono::Duration::hours(5),
        },
        // Development - In Progress
        tasks::Model {
            id: 3,
            asana_id: "DEMO-003".to_string(),
            title: "Implement WASM Build".to_string(),
            description: Some("Fix compilation errors and get trunk build working".to_string()),
            status: "In Progress".to_string(),
            priority: Some("high".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(1)),
            github_pr_url: None,
            review_status: None,
            last_updated_at: Local::now().naive_local(),
        },
        tasks::Model {
            id: 4,
            asana_id: "DEMO-004".to_string(),
            title: "Add UI Animations".to_string(),
            description: Some(
                "Implement smooth transitions and effects using tachyonfx".to_string(),
            ),
            status: "In Progress".to_string(),
            priority: Some("medium".to_string()),
            due_date: None,
            github_pr_url: None,
            review_status: None,
            last_updated_at: Local::now().naive_local() - chrono::Duration::hours(2),
        },
        tasks::Model {
            id: 5,
            asana_id: "DEMO-005".to_string(),
            title: "Refactor Storage Layer".to_string(),
            description: Some("Abstract storage interface for better testability".to_string()),
            status: "In Progress".to_string(),
            priority: Some("low".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(5)),
            github_pr_url: None,
            review_status: None,
            last_updated_at: Local::now().naive_local() - chrono::Duration::days(1),
        },
        // Internal Review - UnChecked
        tasks::Model {
            id: 6,
            asana_id: "PR-42".to_string(),
            title: "Code Review: Authentication Module".to_string(),
            description: Some("Review security improvements and token handling".to_string()),
            status: "Internal Review UnChecked".to_string(),
            priority: Some("high".to_string()),
            due_date: Some(Local::now().naive_local()),
            github_pr_url: Some("https://github.com/example/repo/pull/42".to_string()),
            review_status: Some(
                serde_json::json!({"status": "pending", "reviewers": ["alice", "bob"]}),
            ),
            last_updated_at: Local::now().naive_local() - chrono::Duration::hours(3),
        },
        tasks::Model {
            id: 7,
            asana_id: "PR-55".to_string(),
            title: "Feature: Export to CSV".to_string(),
            description: Some("Add data export functionality for reports".to_string()),
            status: "Internal Review UnChecked".to_string(),
            priority: Some("medium".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(2)),
            github_pr_url: Some("https://github.com/example/repo/pull/55".to_string()),
            review_status: Some(serde_json::json!({"status": "pending", "reviewers": ["charlie"]})),
            last_updated_at: Local::now().naive_local() - chrono::Duration::hours(8),
        },
        // Internal Review - Checked
        tasks::Model {
            id: 8,
            asana_id: "PR-38".to_string(),
            title: "Bugfix: Calendar Date Selection".to_string(),
            description: Some("Fix timezone handling in calendar widget".to_string()),
            status: "Internal Review Checked".to_string(),
            priority: Some("high".to_string()),
            due_date: Some(Local::now().naive_local() - chrono::Duration::days(1)),
            github_pr_url: Some("https://github.com/example/repo/pull/38".to_string()),
            review_status: Some(
                serde_json::json!({"status": "approved", "reviewers": ["alice"], "comments": 2}),
            ),
            last_updated_at: Local::now().naive_local() - chrono::Duration::days(2),
        },
        // External Review - UnChecked
        tasks::Model {
            id: 9,
            asana_id: "PR-61".to_string(),
            title: "API Integration: GitHub Sync".to_string(),
            description: Some("Sync tasks from GitHub issues and PRs".to_string()),
            status: "External Review UnChecked".to_string(),
            priority: Some("high".to_string()),
            due_date: Some(Local::now().naive_local() + chrono::Duration::days(1)),
            github_pr_url: Some("https://github.com/example/repo/pull/61".to_string()),
            review_status: Some(
                serde_json::json!({"status": "pending", "external_reviewers": ["partner-team"]}),
            ),
            last_updated_at: Local::now().naive_local() - chrono::Duration::hours(12),
        },
        // External Review - Checked
        tasks::Model {
            id: 10,
            asana_id: "PR-47".to_string(),
            title: "Documentation: API Reference".to_string(),
            description: Some("Complete API documentation with examples".to_string()),
            status: "External Review Checked".to_string(),
            priority: Some("medium".to_string()),
            due_date: None,
            github_pr_url: Some("https://github.com/example/repo/pull/47".to_string()),
            review_status: Some(
                serde_json::json!({"status": "approved", "external_reviewers": ["docs-team"], "comments": 5}),
            ),
            last_updated_at: Local::now().naive_local() - chrono::Duration::days(3),
        },
    ]
}
