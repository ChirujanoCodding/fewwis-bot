use crate::{
    api::{Translator, RAPID_API_TRANSLATE},
    Context, Error,
};
use ::serenity::{builder::CreateEmbed, model::Color};
use poise::{serenity_prelude as serenity, CreateReply};
use reqwest::header;

/// Translate a text
#[poise::command(context_menu_command = "Translate text", category = "Utilities")]
pub async fn translate_ctx_menu(
    ctx: Context<'_>,
    #[description = "The text to translate"] msg: serenity::Message,
) -> Result<(), Error> {
    let text = msg.content.to_string();
    let response = get_translation(text.clone()).await?;

    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    ctx.send(
        reply.embed(
            embed
                .title("📄 Google translate (joke i don't have money to pay that)")
                .fields([
                    ("📤 Original Text: ", text, false),
                    (
                        "✨ **Translated Text**",
                        response.data.translated_text,
                        false,
                    ),
                ])
                .color(Color::BLURPLE),
        ),
    )
    .await?;
    Ok(())
}

/// Translate a text
#[poise::command(slash_command, category = "Utilities")]
pub async fn translate(
    ctx: Context<'_>,
    #[description = "The text to translate"] text: String,
) -> Result<(), Error> {
    let response = get_translation(text.clone()).await?;

    let reply = CreateReply::default();
    let embed = CreateEmbed::new();

    ctx.send(
        reply.embed(
            embed
                .title("📄 Google translate (joke i don't have money to pay that)")
                .fields([
                    ("📤 Original Text: ", text, false),
                    (
                        "✨ **Translated Text**",
                        response.data.translated_text,
                        false,
                    ),
                ])
                .color(Color::BLURPLE),
        ),
    )
    .await?;
    Ok(())
}

async fn get_translation(text: String) -> Result<Translator, Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "content-type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    headers.insert(
        "X-RapidAPI-Key",
        dotenvy::var("RAPID_API_KEY").unwrap().parse().unwrap(),
    );
    headers.insert(
        "X-RapidAPI-Host",
        "text-translator2.p.rapidapi.com".parse().unwrap(),
    );

    let client = reqwest::Client::new();
    Ok(client
        .post(RAPID_API_TRANSLATE)
        .headers(headers)
        .form(&[
            ("source_language", "en"),
            ("target_language", "es"),
            ("text", &text.clone()),
        ])
        .send()
        .await?
        .json::<Translator>()
        .await?)
}
