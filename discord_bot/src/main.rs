mod dalle_commands;

use dalle::Dalle;
use log::{info, LevelFilter};
use poise::serenity_prelude as serenity;
use poise::FrameworkOptions;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

pub type DiscordContext<'a> = poise::Context<'a, Data<Dalle>, anyhow::Error>;
pub const IMAGE_DIR: &str = "images";

#[derive(Deserialize, Serialize)]
struct Config {
    discord_token: String,
    dalle_token: String,
    number_of_polls: usize
}
pub struct Data<TDalle: dalle::DalleClient> {
    dalle: TDalle,
}

#[poise::command(prefix_command)]
async fn register(ctx: DiscordContext<'_>) -> anyhow::Result<()> {
    info!("Got register command");
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let config = load_config().await;
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("serenity", LevelFilter::Warn)
        .with_module_level("tracing::span", LevelFilter::Warn)
        .init()
        .unwrap();

    info!("Tokens retrieved, starting bot.");

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands: vec![register(), dalle_commands::dalle_generate(), dalle_commands::get_credits()],
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    dalle: Dalle::new(&config.dalle_token, config.number_of_polls).expect("Expect Dalle to be created"),
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
