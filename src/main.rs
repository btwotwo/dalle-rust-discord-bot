use std::{fs};
pub mod dalle;
use futures::executor::block_on;
use serde_json::Value;
use text_io::{scan, read};

fn main() {
    block_on(async_main()).unwrap()
}

async fn async_main() -> anyhow::Result<()>{
    let token = ger_token_from_config()?;
    let dalle = dalle::Dalle::new(&token);
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
    Ok(config_json["token"].to_string())
}
