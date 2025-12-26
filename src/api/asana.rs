use anyhow::Result;
use serde_json::Value;
use std::time::Duration;

pub struct AsanaClient {
    client: reqwest::Client,
    token: String,
}

impl AsanaClient {
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, token }
    }

    pub async fn search_tasks(
        &self,
        workspace_gid: &str,
        assignee_gid: Option<&str>,
    ) -> Result<Value> {
        let mut url = format!(
            "https://app.asana.com/api/1.0/workspaces/{}/tasks/search?completed=false&opt_fields=gid,name,notes,due_on,resource_subtype,custom_fields,custom_fields.name,custom_fields.display_value,custom_fields.enum_value.name",
            workspace_gid
        );

        // Add assignee filter if provided
        if let Some(assignee) = assignee_gid {
            url.push_str(&format!("&assignee.any={}", assignee));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Asana API error {}: {}", status, body);
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn get_task(&self, task_gid: &str) -> Result<Value> {
        let url = format!("https://app.asana.com/api/1.0/tasks/{}", task_gid);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Asana API error {}: {}", status, body);
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    pub async fn get_task_stories(&self, task_gid: &str) -> Result<Value> {
        let url = format!(
            "https://app.asana.com/api/1.0/tasks/{}/stories?opt_fields=text,created_by.name",
            task_gid
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Asana API error {}: {}", status, body);
        }

        let json: Value = response.json().await?;
        Ok(json)
    }
}
