#[cfg(not(target_arch = "wasm32"))]
use crate::api::asana::AsanaClient;
#[cfg(not(target_arch = "wasm32"))]
use crate::api::github::GitHubClient;
use crate::db::tasks;
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::Value;

#[cfg(not(target_arch = "wasm32"))]
use crate::logic::reviewers::ReviewerConfig;

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// Lazy-compiled regex patterns for performance
#[cfg(not(target_arch = "wasm32"))]
static PR_URL_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"https://github\.com/([^/]+)/([^/]+)/pull/(\d+)").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerState {
    pub name: String,
    pub status: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskReviewStatus {
    pub internal: Vec<ReviewerState>,
    pub external: Vec<ReviewerState>,
}

#[derive(Debug, Default)]
#[cfg(not(target_arch = "wasm32"))]
pub struct GitHubPrData {
    pub has_internal_reviewer: bool,
    pub internal_reviews_finished: bool,
    pub has_external_reviewer: bool,
    pub external_reviews_finished: bool,
    pub review_status: Option<TaskReviewStatus>,
}

pub struct TaskSynchronizer {
    #[cfg(not(target_arch = "wasm32"))]
    asana_client: Option<AsanaClient>,
    #[cfg(not(target_arch = "wasm32"))]
    github_client: Option<GitHubClient>,
    #[cfg(not(target_arch = "wasm32"))]
    reviewer_config: Option<ReviewerConfig>,
}

impl Default for TaskSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSynchronizer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        let reviewer_config = ReviewerConfig::load("reviewers.json").ok();

        let asana_client = std::env::var("ASANA_ACCESS_TOKEN")
            .ok()
            .map(AsanaClient::new);

        let github_client = std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN")
            .ok()
            .map(GitHubClient::new);

        Self {
            asana_client,
            github_client,
            reviewer_config,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn sync(&self) -> Result<Vec<tasks::Model>> {
        Ok(Vec::new())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn parse_asana_tasks(
        &self,
        data: &Value,
        log: &mut std::fs::File,
    ) -> Result<Vec<tasks::Model>> {
        use std::io::Write;
        let mut parsed_tasks = Vec::new();

        // Asana API returns: {"data": [...tasks...]}
        if let Some(tasks_array) = data.get("data").and_then(|v| v.as_array()) {
            for task_obj in tasks_array {
                let asana_id = task_obj
                    .get("gid")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                let name = task_obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string();

                let notes_str = task_obj.get("notes").and_then(|v| v.as_str()).unwrap_or("");

                // Parse task type from resource_subtype
                let task_type = task_obj
                    .get("resource_subtype")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default_task");

                let description = if !notes_str.is_empty() {
                    Some(format!("[Type: {}] {}", task_type, notes_str))
                } else {
                    Some(format!("[Type: {}]", task_type))
                };

                let due_date = task_obj
                    .get("due_on")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap());

                let status = "Imported".to_string();

                let task = tasks::Model {
                    id: 0,
                    asana_id,
                    title: name,
                    description,
                    status,
                    due_date,
                    priority: None,
                    github_pr_url: None,
                    review_status: None,
                    last_updated_at: chrono::Utc::now().naive_utc(),
                };

                let mut asana_status = None;
                let mut task_category = None;

                if let Some(custom_fields) = task_obj.get("custom_fields") {
                    writeln!(log, "[SYNC] Task '{}' custom fields:", task.title).ok();
                    if let Some(fields_array) = custom_fields.as_array() {
                        for field in fields_array {
                            let field_name = field
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown");
                            let display_value = field
                                .get("display_value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("N/A");
                            writeln!(log, "[SYNC]   - {}: {}", field_name, display_value).ok();

                            if field_name == "ステータス" && display_value != "N/A" {
                                asana_status = Some(display_value.to_string());
                            } else if field_name == "タスク種別" && display_value != "N/A" {
                                task_category = Some(display_value.to_string());
                            }
                        }
                    }
                }

                let mut task = task;
                if let Some(status) = asana_status {
                    writeln!(
                        log,
                        "[SYNC] Using Asana status '{}' for task '{}'",
                        status, task.title
                    )
                    .ok();
                    task.status = status;
                }

                if let Some(category) = task_category {
                    task.priority = Some(category);
                }

                parsed_tasks.push(task);
            }
        }

        Ok(parsed_tasks)
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn extract_pr_url_from_comments(&self, task_gid: &str) -> Option<String> {
        let Some(asana) = &self.asana_client else {
            return None;
        };

        let stories = asana.get_task_stories(task_gid).await.ok()?;
        let stories_array = stories.get("data")?.as_array()?;

        for story in stories_array {
            if let Some(text) = story.get("text").and_then(|t| t.as_str()) {
                if text.contains("PRを作成しました") {
                    if let Some(caps) = PR_URL_REGEX.captures(text) {
                        return Some(caps.get(0)?.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn process_github_pr(&self, url: &str) -> Result<GitHubPrData> {
        let Some(github) = &self.github_client else {
            anyhow::bail!("GitHub client not available");
        };

        let caps = PR_URL_REGEX
            .captures(url)
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub PR URL"))?;

        let owner = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str();
        let pr_number: u64 = caps.get(3).unwrap().as_str().parse()?;

        let pr_data = github.get_pull_request(owner, repo, pr_number).await?;
        let reviews_data = github.get_pr_reviews(owner, repo, pr_number).await?;

        self.check_reviewers(&pr_data, &reviews_data).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn check_reviewers(&self, pr_data: &Value, reviews_data: &Value) -> Result<GitHubPrData> {
        let config = self
            .reviewer_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Reviewer config not loaded"))?;

        let mut data = GitHubPrData::default();
        let mut review_status = TaskReviewStatus::default();

        let requested_reviewers: Vec<String> = pr_data
            .get("requested_reviewers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r.get("login").and_then(|l| l.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let reviews = reviews_data
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid reviews data"))?;

        let check_reviewer = |username: &str| -> ReviewerState {
            let user_review = reviews
                .iter()
                .filter(|r| {
                    r.get("user")
                        .and_then(|u| u.get("login"))
                        .and_then(|l| l.as_str())
                        == Some(username)
                })
                .next_back();

            let mut status = if requested_reviewers.contains(&username.to_string()) {
                "Pending".to_string()
            } else {
                "Not Involved".to_string()
            };

            let mut url = None;

            if let Some(review) = user_review {
                if let Some(state) = review.get("state").and_then(|s| s.as_str()) {
                    status = match state {
                        "APPROVED" => "Approved".to_string(),
                        "CHANGES_REQUESTED" => "Changes Requested".to_string(),
                        "COMMENTED" => "Commented".to_string(),
                        "DISMISSED" => "Dismissed".to_string(),
                        _ => state.to_string(),
                    };
                    url = review
                        .get("html_url")
                        .and_then(|u| u.as_str())
                        .map(String::from);
                }
            }

            ReviewerState {
                name: username.to_string(),
                status,
                url,
            }
        };

        let required_internal_reviewers: Vec<_> = config
            .internal_reviewers
            .iter()
            .filter(|r| requested_reviewers.contains(r))
            .collect();
        data.has_internal_reviewer = !required_internal_reviewers.is_empty();

        let mut internal_approved_count = 0;
        for internal in &config.internal_reviewers {
            let state = check_reviewer(internal);
            if state.status == "Approved" && required_internal_reviewers.contains(&internal) {
                internal_approved_count += 1;
            }
            review_status.internal.push(state);
        }
        data.internal_reviews_finished = data.has_internal_reviewer
            && internal_approved_count == required_internal_reviewers.len();

        let required_external_reviewers: Vec<_> = config
            .external_reviewers
            .iter()
            .filter(|r| requested_reviewers.contains(r))
            .collect();
        data.has_external_reviewer = !required_external_reviewers.is_empty();

        let mut external_approved_count = 0;
        for external in &config.external_reviewers {
            let state = check_reviewer(external);
            if state.status == "Approved" && required_external_reviewers.contains(&external) {
                external_approved_count += 1;
            }
            review_status.external.push(state);
        }
        data.external_reviews_finished = data.has_external_reviewer
            && external_approved_count == required_external_reviewers.len();

        data.review_status = Some(review_status);
        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn infer_status(&self, asana_status: &str, github_data: Option<&GitHubPrData>) -> String {
        match asana_status {
            "Ready" => "Not Started".to_string(),
            "In Progress" => "In Progress".to_string(),
            "In Review" => {
                if let Some(gh) = github_data {
                    if gh.has_external_reviewer {
                        if gh.external_reviews_finished {
                            "External Review Checked".to_string()
                        } else {
                            "External Review UnChecked".to_string()
                        }
                    } else if gh.has_internal_reviewer || gh.internal_reviews_finished {
                        if gh.internal_reviews_finished {
                            "Internal Review Checked".to_string()
                        } else {
                            "Internal Review UnChecked".to_string()
                        }
                    } else {
                        "In Review".to_string()
                    }
                } else {
                    "In Review".to_string()
                }
            }
            "Imported" | "Not Started" | "" => "Not Started".to_string(),
            other => other.to_string(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn sync(&self) -> Result<Vec<tasks::Model>> {
        use std::io::Write;

        if let Err(e) = std::fs::create_dir_all("logs") {
            eprintln!("Warning: Failed to create logs directory: {}", e);
        }

        let mut log = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/sync_detail.log")
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Warning: Failed to open sync detail log file: {}. Continuing without detailed logging.", e);
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/null")
                    .unwrap_or_else(|_| {
                        std::fs::File::create("/tmp/taskmanager_dummy.log").unwrap()
                    })
            }
        };

        writeln!(log, "\n[SYNC] Starting sync process").ok();

        let Some(asana) = &self.asana_client else {
            writeln!(log, "[SYNC] No Asana client available").ok();
            return Ok(Vec::new());
        };

        let workspace = std::env::var("ASANA_WORKSPACE_GID")?;
        let assignee = std::env::var("ASANA_USER_GID").ok();

        writeln!(log, "[SYNC] Calling Asana API: workspace={}", workspace).ok();

        let result = asana.search_tasks(&workspace, assignee.as_deref()).await?;
        writeln!(log, "[SYNC] Asana API response received").ok();

        let mut tasks = self.parse_asana_tasks(&result, &mut log)?;
        writeln!(log, "[SYNC] Parsed {} tasks", tasks.len()).ok();

        for task in &mut tasks {
            let mut github_data = None;
            let mut has_pr = false;

            if let Some(url) = self.extract_pr_url_from_comments(&task.asana_id).await {
                writeln!(log, "[SYNC] Found PR in comments: {}", url).ok();
                has_pr = true;
                if let Ok(data) = self.process_github_pr(&url).await {
                    github_data = Some(data);
                }
            }

            let inferred = if has_pr {
                if let Some(gh) = github_data.as_ref() {
                    if gh.has_external_reviewer {
                        if gh.external_reviews_finished {
                            "External Review Checked".to_string()
                        } else {
                            "External Review UnChecked".to_string()
                        }
                    } else if gh.internal_reviews_finished {
                        "Internal Review Checked".to_string()
                    } else {
                        "Internal Review UnChecked".to_string()
                    }
                } else {
                    "Internal Review UnChecked".to_string()
                }
            } else {
                self.infer_status(&task.status, None)
            };

            writeln!(
                log,
                "[SYNC] Task '{}': {} (has_pr: {})",
                task.title, inferred, has_pr
            )
            .ok();
            task.status = inferred;

            if let Some(gh) = github_data {
                if let Some(rs) = gh.review_status {
                    task.review_status = serde_json::to_value(rs).ok();
                }
            }
        }

        let all_tasks = tasks;
        writeln!(log, "[SYNC] Total tasks: {}", all_tasks.len()).ok();

        writeln!(log, "[SYNC] Sync completed successfully").ok();
        Ok(all_tasks)
    }
}
