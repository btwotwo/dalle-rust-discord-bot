

use anyhow::{Context, anyhow};
use const_format::concatcp;
use reqwest::{Client, header::{CONTENT_TYPE, AUTHORIZATION}};
use serde_json::{Value};
const BASE_URL: &str = "https://labs.openai.com/api/labs";
const TASKS_URL: &str = concatcp!(BASE_URL, "/tasks");


pub struct Dalle {
    client: Client
}

pub struct DalleResponse {
    pub image_url: String,
}

impl Dalle {
    pub fn new(token: &str) -> anyhow::Result<Self> {
        use reqwest::ClientBuilder;
        use reqwest::header::{HeaderMap, HeaderValue};
        let formatted_token = format!("Bearer {}", token);
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut auth_header_value = HeaderValue::from_str(&formatted_token).context("expected token to be valid header string")?;
        auth_header_value.set_sensitive(true);
        default_headers.insert(AUTHORIZATION, auth_header_value);

        let client = ClientBuilder::new().default_headers(default_headers).build()?;
        Ok(Self {client})
    }

    pub async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>> {
        use tokio::time::sleep;
        use tokio::time::Duration;
        const MAX_ATTEMPTS: u8 = 10;

        let body = format!(
            r#"
            {{
                "task_type": "text2im",
                "prompt": {{
                    "caption": "{}",
                    "batch_size": 4
                }}
            }}
        "#,
            prompt
        );

        let request = self.client.post(TASKS_URL).body(body);
        let res = request.send().await?;
        let res_json: Value  = res.json().await?;
        let task_id = res_json["id"].as_str().with_context(|| format!("expected task id to be string, but instead got {}", res_json))?;
        let mut attempts = 0;

        while attempts <= MAX_ATTEMPTS {
            attempts += 1;
            let task = self.get_task(task_id).await?;
            if let Some(gens) = task {
                return Ok(gens);
            } else {
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        }

        Err(anyhow!("Didn't get result after {} attempts", MAX_ATTEMPTS))
    }

    pub async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Vec<DalleResponse>>> {
        let task_url = format!("{}/{}", TASKS_URL, task_id);
        let res: Value = self.client.get(task_url).send().await?.json().await?;
        let task_status = res["status"].as_str().with_context(|| format!("expected res to have task status, but got {}", res))?;
        match task_status {
            "succeeded" => {
                let gens = res["generations"].as_array().with_context(|| format!("expected result to have generations, but got {}", res))?.iter().map(|gen| DalleResponse {
                    image_url: gen["generation"]["image_path"].as_str().unwrap().to_string()
                }).collect();
                return Ok(Some(gens))
            }
            "rejected" => {
                return Err(anyhow!("Generation is rejected. Full response: {}", res))
            }
            "pending" => {
                return Ok(None)
            }
            _ => {return Err(anyhow!("Invalid task status: {}", task_status))}
        }
    }
}