// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use reqwest::{header, Client};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RunnerList {
    pub runners: Vec<Runner>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Runner {
    pub id: u64,
    pub name: String,
    pub busy: bool,
}

pub struct GitHubClient {
    base: String,
    client: Client,
}

impl GitHubClient {
    pub fn new(token: String, org: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("github-runner-tui"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("token {}", token)).unwrap(),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        let parts: Vec<&str> = org.split('/').collect();

        let base = if parts.len() > 1 {
            format!("repos/{}/{}", parts[0], parts[1])
        } else {
            format!("orgs/{}", parts[0])
        };

        Self { base, client }
    }

    pub async fn runners(&self) -> Result<RunnerList, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://api.github.com/{}/actions/runners", &self.base);

        let res = self.client.get(url).send().await?.error_for_status()?;

        let mut runners: RunnerList = res.json::<RunnerList>().await?;
        runners.runners.sort_by_key(|r| r.id);

        return Ok(runners);
    }

    pub async fn delete_self_hosted_runner(
        &self,
        runner_id: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/{}/actions/runners/{}",
            &self.base, runner_id
        );

        self.client.delete(&url).send().await?.error_for_status()?;

        Ok(())
    }
}
