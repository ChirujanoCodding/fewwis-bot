use crate::{Context, Error};
use ::serenity::builder::CreateEmbed;
use poise::{serenity_prelude as serenity, CreateReply};

/// Get the avatar of any member in the server
#[poise::command(context_menu_command = "Get avatar", category = "Utilities")]
pub async fn avatar_ctx_menu(
    ctx: Context<'_>,
    #[description = "The member to fetch"] msg: serenity::Message,
) -> Result<(), Error> {
    let user = msg.author;

    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    ctx.send(
        reply.embed(
            embed
                .title(format!("🎭 Avatar of `{}`", user.name))
                .image(user.avatar_url().unwrap()),
        ),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, category = "Utilities")]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "The member to fetch"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.unwrap_or(ctx.author().clone());
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    ctx.send(
        reply.embed(
            embed
                .title(format!("🎭 Avatar of `{}`", user.name))
                .image(user.avatar_url().unwrap()),
        ),
    )
    .await?;
    Ok(())
}
