use std::time::Duration;

use crate::{
    helper::db::delete_post,
    types::{Context, Error},
};
use ::serenity::{builder::CreateEmbed, model::Color};
use poise::{serenity_prelude as serenity, CreateReply};
use tracing::error;

/// Delete a task
#[poise::command(slash_command, check = "super::is_forum_post", category = "Utilities")]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Skip the button confirmation."] force: Option<bool>,
) -> Result<(), Error> {
    let Ok(serenity::Channel::Guild(channel)) = ctx.channel_id().to_channel(ctx).await else {
        error!("❌ Cannot fetch the guild channel...");
        return Err("Error fetching guild channel.".into());
    };

    let force = force.unwrap_or_default();

    let db = &ctx.data().db;
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();
    let row = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("confirm")
            .style(serenity::ButtonStyle::Danger)
            .label("Yes")
            .emoji('✅'),
        serenity::CreateButton::new("cancel")
            .style(serenity::ButtonStyle::Secondary)
            .label("No")
            .emoji('❌'),
    ]);

    if !force {
        let response = ctx
            .send(
                reply
                    .clone()
                    .embed(
                        embed
                            .clone()
                            .color(Color::RED)
                            .title("🤚 Wait. Are you sure to delete the task?")
                            .description("> This operation is not reversible."),
                    )
                    .components(vec![row])
                    .ephemeral(true),
            )
            .await?;
        if let Some(interaction) = response
            .message()
            .await?
            .await_component_interactions(ctx.serenity_context().shard.clone())
            .author_id(ctx.author().id)
            .message_id(response.message().await?.id)
            .timeout(Duration::from_secs(60))
            .await
        {
            match interaction.data.custom_id.as_str() {
                "cancel" => {
                    response.delete(ctx).await?;
                }
                "confirm" => {
                    delete_post(db, channel.id.get()).await?;
                    channel.delete(ctx).await?;
                }
                _ => unreachable!(),
            };
        } else {
            response.edit(ctx, reply
                .embed(
                    embed
                        .color(Color::RED)
                        .title("⌛ Oops... Timeout expired!")
                        .description("> You did not interact with any button, so the action was cancelled."),
                ).components(vec![])
                .ephemeral(true)).await?;
        }
    } else {
        delete_post(db, channel.id.get()).await?;
        channel.delete(ctx).await?;
    }

    Ok(())
}
