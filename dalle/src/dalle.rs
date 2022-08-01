mod dalle_trait;
use anyhow::{anyhow, Context};
use const_format::concatcp;
use log::{info, warn};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde_json::Value;
pub use dalle_trait::DalleGenerator;
use async_trait::async_trait;
const BASE_URL: &str = "https://labs.openai.com/api/labs";
const TASKS_URL: &str = concatcp!(BASE_URL, "/tasks");

pub struct Dalle {
    client: Client,
}

pub struct DalleResponse {
    pub image_url: String,
}

#[async_trait]
impl DalleGenerator for Dalle {
    async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>> {
        self.generate(prompt).await
    }

    async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Vec<DalleResponse>>> {
        self.get_task(task_id).await
    }
}

impl Dalle {
    pub fn new(token: &str) -> anyhow::Result<Self> {
        use reqwest::header::{HeaderMap, HeaderValue};
        use reqwest::ClientBuilder;
        let formatted_token = format!("Bearer {}", token);
        let mut default_headers = HeaderMap::new();
        default_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut auth_header_value = HeaderValue::from_str(&formatted_token)
            .context("expected token to be valid header string")?;
        auth_header_value.set_sensitive(true);
        default_headers.insert(AUTHORIZATION, auth_header_value);

        let client = ClientBuilder::new()
            .default_headers(default_headers)
            .build()?;
        Ok(Self { client })
    }

    pub async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>> {
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

        info!("Sending following request to dalle: {}", body);

        let request = self.client.post(TASKS_URL).body(body);
        let res = request.send().await?;
        info!("Got response from dalle: {:?}", res);

        let res_json: Value = res.json().await?;

        let task_id = res_json["id"].as_str().with_context(|| {
            format!(
                "expected task id to be string, but instead got {}",
                res_json
            )
        })?;

        Ok(self.wait_for_generation_completion(task_id).await?)
    }

    pub async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Vec<DalleResponse>>> {
        let task_url = format!("{}/{}", TASKS_URL, task_id);
        let res: Value = self.client.get(task_url).send().await?.json().await?;
        
        info!("Got task info: {:?}", res);

        let task_status = res["status"]
            .as_str()
            .with_context(|| format!("expected res to have task status, but got {}", res))?;

        match task_status {
            "succeeded" => {
                info!("Task succeeded, collecting generations.");
                let generations = Dalle::collect_generations(res)?;
                Ok(Some(generations))
            }
            "rejected" => {warn!("Task rejected, full response: {}", res); return Err(anyhow!("Generation is rejected. Full response: {}", res))},
            "pending" => return Ok(None),
            _ => return Err(anyhow!("Invalid task status: {}", task_status)),
        }
    }

    fn collect_generations(res: Value) -> anyhow::Result<Vec<DalleResponse>> {
        let gens = res["generations"]["data"]
            .as_array()
            .with_context(|| format!("expected result to have generations, but got {}", res))?
            .iter()
            .map(|gen| DalleResponse {
                image_url: gen["generation"]["image_path"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            })
            .collect();
        Ok(gens)
    }

    async fn wait_for_generation_completion(
        &self,
        task_id: &str,
    ) -> anyhow::Result<Vec<DalleResponse>> {
        use tokio::time::sleep;
        use tokio::time::Duration;

        const MAX_ATTEMPTS: u8 = 10;
        let mut attempts = 0;

        while attempts <= MAX_ATTEMPTS {
            attempts += 1;
            let task = self.get_task(task_id).await;

            match task {
                Ok(Some(gens)) => return Ok(gens),
                Ok(None) => {
                    sleep(Duration::from_secs(2)).await;
                    continue;
                },
                Err(e) => {
                    warn!("Got error while downloading image, retrying: {}", e);
                    continue;
                }
            }
        }

        Err(anyhow!("Didn't get result after {} attempts", MAX_ATTEMPTS))
    }
}
