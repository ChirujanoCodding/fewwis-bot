use ::poise::CreateReply;
use poise::serenity_prelude as serenity;
use poise::ChoiceParameter;
use serenity::all::ForumTagId;
use serenity::builder::CreateEmbed;
use serenity::builder::EditThread;
use serenity::model::Color;
use tracing::error;

use crate::types::Context;
use crate::types::Error;

#[repr(u64)]
#[derive(ChoiceParameter, Clone, Copy, PartialEq)]
pub enum Status {
    #[name = "✏ To-do"]
    Todo = 1150602743623471215,
    #[name = "⏳ Working on it"]
    Working = 1150602817585815603,
    #[name = "✅ Finished"]
    Finished = 1150602652904853564,
}

/// Change the status of any task
#[poise::command(slash_command, check = "super::is_forum_post", category = "Utilities")]
pub async fn update(
    ctx: Context<'_>,
    #[description = "the new tag to apply to the task"] status: Status,
) -> Result<(), Error> {
    let Ok(serenity::Channel::Guild(mut channel)) = ctx.channel_id().to_channel(ctx).await else {
        error!("❌ Cannot fetch the guild channel...");
        return Err("Error fetching guild channel.".into());
    };

    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    let forum_tag_id = ForumTagId::from(status as u64);

    if channel.applied_tags.contains(&forum_tag_id) {
        return Err("You can't set the same tag.".into());
    }

    ctx.send(
        reply
            .embed(embed.color(Color::FOOYOO).title("✅ To-do updated!"))
            .ephemeral(true),
    )
    .await?;

    channel
        .edit_thread(
            ctx,
            EditThread::default()
                .applied_tags([forum_tag_id])
                .archived(status == Status::Finished),
        )
        .await?;
    Ok(())
}
