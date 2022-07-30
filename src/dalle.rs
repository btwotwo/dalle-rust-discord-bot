use std::any;

use anyhow::Context;
use reqwest::Client;
use serde_json::{Result, Value};
const URL: &'static str = "https://labs.openai.com/api/labs/tasks";

pub struct Dalle<'a> {
    token: &'a str,
    client: Client
}

pub struct DalleResponse {
    pub image_url: String,
}

impl<'a> Dalle<'a> {
    pub fn new(token: &'a str) -> Self {
        Self { token, client: Client::new() }
    }

    pub async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>> {
        let body = format!(
            r#"
            {{
                task_type: "text2im",
                prompt: {{
                    caption: {},
                    batch_size: 4
                }}
            }}
        "#,
            prompt
        );
        let auth = format!("Bearer {}", self.token);
        let res = self.client.post(URL).body(body).header("Authorization", auth).send().await?;
        let res_json: Value  = serde_json::from_str(&res.text().await?)?;
        let res_json = res_json.as_array().context("response from Dalle should be an array")?;

        Ok(res_json.iter().map(|item| DalleResponse {
            image_url: item["generation"]["image_path"].to_string()
        }).collect())
    }
}
