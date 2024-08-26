use std::time::Instant;

use crate::{helper::Colors, Context, Error};
use ::serenity::builder::{CreateAllowedMentions, CreateEmbed};
use poise::CreateReply;

/// Pong!
#[poise::command(slash_command, category = "Utilities")]
pub async fn ping(
    ctx: Context<'_>,
    #[description = "Make the message ephemeral?"] ephemeral: Option<bool>,
) -> Result<(), Error> {
    let shard_manager = ctx.framework().shard_manager();

    let before = Instant::now();
    let runners = shard_manager.runners.lock().await;

    let runner = runners
        .get(&ctx.serenity_context().shard_id)
        .ok_or("❌ Cannot found shard.")?;

    let ping = runner
        .latency
        .unwrap_or(Instant::now().duration_since(before));

    let color = match ping.as_millis() {
        u128::MIN => Colors::Gray,
        1..100 => Colors::Green,
        100..200 => Colors::Orange,
        _ => Colors::Red,
    };

    let db = &ctx.data().db;

    let before = Instant::now();
    db.ping().await?;
    let after = Instant::now();
    let database_ping = after.duration_since(before);

    let reply = CreateReply::default();
    let embed = CreateEmbed::new();
    ctx.send(
        reply
            .embed(
                embed
                    .title("🏓 Pong")
                    .description(format!(
                        "> 📡 Discord ping: `{ping:?}`\n> 📡 Database ping: `{database_ping:?}`"
                    ))
                    .color(color),
            )
            .allowed_mentions(CreateAllowedMentions::new().empty_users())
            .ephemeral(ephemeral.unwrap_or_default()),
    )
    .await?;

    Ok(())
}
