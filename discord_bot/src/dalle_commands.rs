use std::borrow::Cow;

use crate::DiscordContext;
use anyhow::Context;
use bytes::Bytes;
use dalle::DalleResponse;
use log::info;
use poise::serenity_prelude::AttachmentType;
use uuid::Uuid;

#[poise::command(slash_command, guild_only)]
pub async fn dalle_generate(
    context: DiscordContext<'_>,
    #[description = "A prompt for DALL-E 2"] prompt: String,
) -> anyhow::Result<()> {
    info!("Got Dalle Generate command with prompt {}", prompt);

    let msg = context
        .say(format!("Sending \"{}\" to DALL-E...", prompt))
        .await?;
    context.defer().await?;

    let dalle = context.data().dalle.generate(&prompt).await?;
    info!("Got responses from DALL-E, downloading them.");

    msg.edit(context, |f| f.content("Got your generation, downloading...")).await?;

    let downloaded_images = download_images_to_fs(dalle, context).await?;
    let attachment_images: Vec<AttachmentType<'_>> = downloaded_images
        .iter()
        .map(move |file| AttachmentType::Bytes {
            data: Cow::Borrowed(&file.content),
            filename: file.filename.clone(),
        })
        .collect();
    
    msg.edit(context, |f| f.content("Images downloaded, uploading them to Discord...")).await?;

    context
        .send(|f| {
            f.attachments = attachment_images;
            f.content(format!("Got your results for \"{}\"", prompt))
        })
        .await?;

    Ok(())
}

struct DalleImageFile {
    content: Bytes,
    filename: String,
}

async fn download_images_to_fs(
    imgs: Vec<DalleResponse>,
    ctx: DiscordContext<'_>,
) -> anyhow::Result<Vec<DalleImageFile>> {
    const IMAGE_COUNT: usize = 4;
    let mut results: Vec<DalleImageFile> = Vec::with_capacity(IMAGE_COUNT);

    for image_data in imgs {
        match download_image(image_data).await {
            Ok(file) => {
                results.push(file);
            }
            Err(e) => {
                ctx.say(format!("Error while downloading file: {}", e))
                    .await
                    .unwrap();
            }
        };
    }
    Ok(results)
}

async fn download_image(img: DalleResponse) -> anyhow::Result<DalleImageFile> {
    info!("Downloading image from {}", img.image_url);

    let filename = format!("{}.webp", Uuid::new_v4());
    let download_url = img.image_url;

    let res = reqwest::get(download_url)
        .await
        .context("Expected image to be downloaded")?;
    let content = res.bytes().await?;

    Ok(DalleImageFile { content, filename })
}
