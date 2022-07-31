use std::{fs};
pub mod dalle;
use anyhow::Context;
use futures::executor::{block_on, self};
use serde_json::Value;
use text_io::{scan, read};

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let token = ger_token_from_config()?;
    let dalle = dalle::Dalle::new(&token)?;
    println!("Enter prompt: ");
    let prompt: String = read!();


    let images = dalle.generate(&prompt).await?;

    for img in images {
        println!("{}", img.image_url)
    }
    Ok(())
}

fn ger_token_from_config() -> anyhow::Result<String> {
    let config = fs::read_to_string("config.json")?;
    let config_json: Value = serde_json::from_str(&config)?;
    let token = config_json["token"].as_str().context("token should be a valid str")?;

    Ok(token.to_string())
}
