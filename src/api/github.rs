use anyhow::Result;
use serde_json::Value;
use std::time::Duration;

pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("TaskManager/1.0")
            .build()
            .expect("Failed to build HTTP client");
        
        Self { client, token }
    }
    
    pub async fn get_pull_request(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Value> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }
        
        let json: Value = response.json().await?;
        Ok(json)
    }
    
    pub async fn get_pr_reviews(&self, owner: &str, repo: &str, pr_number: u64) -> Result<Value> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            owner, repo, pr_number
        );
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }
        
        let json: Value = response.json().await?;
        Ok(json)
    }
}
