use anyhow::Context;
use dalle::Dalle;
use futures::executor::{self, block_on};
use log::info;
use serde_json::Value;
use std::fs;
use text_io::{read, scan};

#[tokio::main]
async fn main() {
    let token = ger_token_from_config().unwrap();
    let dalle = Dalle::new(&token, 10).unwrap();
    println!("Enter prompt: ");
    let prompt: String = read!();

    info!("Generating images...");
    let images = dalle.generate(&prompt).await.unwrap();

    for img in images {
        println!("{}", img.image_url)
    }
}

fn ger_token_from_config() -> anyhow::Result<String> {
    let config = fs::read_to_string("config.json")?;
    let config_json: Value = serde_json::from_str(&config)?;
    let token = config_json["token"]
        .as_str()
        .context("token should be a valid str")?;

    Ok(token.to_string())
}
