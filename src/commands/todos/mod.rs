use poise::serenity_prelude as serenity;
use serenity::all::ChannelType;
use tracing::error;

use crate::{
    consts::OWNER_GUILD,
    types::{Context, Error, FnCommands},
};

mod create;
mod delete;
mod steps;
mod update;

pub fn commands() -> FnCommands {
    vec![create::create_ctx_menu, todo]
}

#[poise::command(
    slash_command,
    subcommands("create::create", "update::update", "delete::delete", "steps::steps"),
    check = "on_private_guild",
    category = "Utilities"
)]
pub async fn todo(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn is_forum_post(ctx: Context<'_>) -> Result<bool, Error> {
    let Ok(serenity::Channel::Guild(channel)) = ctx.channel_id().to_channel(ctx).await else {
        error!("❌ Cannot fetch the guild channel...");
        return Err("Error fetching guild channel".into());
    };

    let Ok(serenity::Channel::Guild(parent_id)) = channel
        .parent_id
        .ok_or("You must be in a forum thread.")?
        .to_channel(ctx)
        .await
    else {
        return Err("You must be in a guild channel.".into());
    };

    if (parent_id.kind != ChannelType::Forum)
        | !matches!(
            channel.kind,
            ChannelType::PublicThread | ChannelType::PrivateThread
        )
    {
        return Err("You must be in a forum post.".into());
    }

    Ok(true)
}

async fn on_private_guild(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.guild_id().unwrap() != OWNER_GUILD {
        Err("Invalid guild".into())
    } else {
        Ok(true)
    }
}
