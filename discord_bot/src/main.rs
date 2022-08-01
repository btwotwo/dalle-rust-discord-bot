mod dalle_commands;
use std::{any, env, fs};

use anyhow::anyhow;
use dalle::Dalle;
use poise::serenity_prelude as serenity;
use poise::{Framework, FrameworkOptions};
use serde::{Deserialize, Serialize};

pub type DiscordContext<'a> = poise::Context<'a, Data, anyhow::Error>;
pub const IMAGE_DIR: &str = "images";

#[derive(Deserialize, Serialize)]
struct Config {
    discord_token: String,
    dalle_token: String,
}
pub struct Data {
    dalle: Dalle,
}

#[poise::command(slash_command)]
async fn test_command(
    ctx: DiscordContext<'_>,
    #[description = "Testing this cool command!"] text: String,
) -> anyhow::Result<()> {
    let response = format!("Hello, {}!", text);
    ctx.say(response).await?;
    match text.as_str() {
        "give me an error!" => Err(anyhow!(":)")),
        _ => {
            ctx.say(format!("MR POGGERS: '{}'", text)).await?;
            Ok(())
        }
    }
}
#[poise::command(prefix_command)]
async fn register(ctx: DiscordContext<'_>) -> anyhow::Result<()> {
    println!("got register command");
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let config = load_config().await;
    tokio::fs::create_dir_all(IMAGE_DIR).await.unwrap();
    poise::Framework::builder()
        .options(FrameworkOptions {
            commands: vec![test_command(), register(), dalle_commands::dalle_generate()],
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    dalle: Dalle::new(&config.dalle_token).expect("Expect Dalle to be created"),
                })
            })
        })
        .run()
        .await
        .unwrap();
}

async fn load_config() -> Config {
    use tokio::fs::read_to_string;
    let config_str = read_to_string("config.json")
        .await
        .expect("Expected config.json");
    let config: Config =
        serde_json::from_str(&config_str).expect("Expected config to be deserialized");
    config
}
