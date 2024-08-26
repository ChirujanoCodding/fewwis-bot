use crate::{
    api::{Memes, MEME_API},
    helper::{Colors, Paginator},
    Context, Error,
};
use ::serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter},
    model::Color,
};
use poise::{serenity_prelude as serenity, CreateReply};

/// Get a meme
#[poise::command(
    slash_command,
    name_localized("es-ES", "memes"),
    description_localized("es-ES", "Obten un meme de forma aleatoria!"),
    category = "Utilities"
)]
pub async fn meme(
    ctx: Context<'_>,
    #[description = "Subreddit to fetch meme"]
    #[name_localized("es-ES", "subreddit")]
    #[description_localized("es-ES", "Subreddit a fetchear")]
    subreddit: Option<String>,
) -> Result<(), Error> {
    let reply = CreateReply::default();
    ctx.defer_ephemeral().await?;
    let initial = ctx
        .send(
            reply.clone().embed(
                CreateEmbed::new()
                    .color(Colors::White)
                    .title("🔁 Fetching memes..."),
            ),
        )
        .await?;

    let memes_count = 50;
    let embeds = get_memes(subreddit, memes_count)
        .await?
        .memes
        .iter()
        .map(|meme| {
            CreateEmbed::new()
                .title(format!("😂 Meme of r/{}", meme.subreddit))
                .description(format!("**{}**", meme.title))
                .url(meme.post_link.clone())
                .footer(CreateEmbedFooter::new(format!(
                    "{} upvotes - By u/{}",
                    meme.ups, meme.author
                )))
                .image(meme.url.clone())
                .color(Color::BLITZ_BLUE)
        })
        .collect::<Vec<_>>();

    let left = CreateButton::new("-5")
        .style(serenity::ButtonStyle::Primary)
        .label("5️⃣")
        .disabled(true);
    let center = CreateButton::new("delete")
        .label("🗑")
        .style(serenity::ButtonStyle::Danger);
    let right = CreateButton::new("+5")
        .style(serenity::ButtonStyle::Primary)
        .label("5️⃣");
    let to_beggining = CreateButton::new("-10")
        .style(serenity::ButtonStyle::Primary)
        .label("🔟")
        .disabled(true);
    let to_final = CreateButton::new("+10")
        .style(serenity::ButtonStyle::Primary)
        .label("🔟");

    let buttons = CreateActionRow::Buttons(vec![to_beggining, left, center, right, to_final]);

    let mut paginator = Paginator::new(embeds.clone()).add_row(
        buttons,
        |_, id, counter, pages| match id {
            "-5" => 0.max(counter - 5),
            "+5" => pages.min(counter + 5),
            "-10" => 0.max(counter - 10),
            "+10" => pages.min(counter + 10),
            _ => counter,
        },
        |id, counter, pages| match id {
            1 | 2 => counter <= 0,
            4 | 5 => counter >= pages - 1,
            _ => false,
        },
    );
    initial.delete(ctx).await?;
    ctx.defer().await?;

    paginator.paginate(ctx).await?;

    Ok(())
}

async fn get_memes(subreddit: Option<String>, count: i64) -> Result<Memes, Error> {
    let url = if let Some(group) = subreddit {
        format!("{MEME_API}{group}/{count}")
    } else {
        format!("{MEME_API}{count}")
    };
    let data = reqwest::get(url).await?.json::<Memes>().await?;
    Ok(data)
}
