use crate::{Context, Error};
use ::serenity::builder::CreateEmbed;
use poise::{serenity_prelude as serenity, CreateReply};

/// Sum 2 numbers
#[poise::command(slash_command, category = "Utilities")]
pub async fn sum(ctx: Context<'_>, a: i32, b: i32) -> Result<(), Error> {
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    ctx.send(
        reply.embed(
            embed
                .title("🧮 Calculator")
                .description(format!("{}", a + b))
                .color(serenity::Color::BLURPLE),
        ),
    )
    .await?;

    Ok(())
}
