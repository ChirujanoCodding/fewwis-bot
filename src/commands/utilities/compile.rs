use crate::{helper::Colors, Context, Error};
use lazy_regex::regex_captures;
use pistones::client::ClientBuilder;

use ::serenity::builder::CreateEmbed;
use poise::{CodeBlock, CreateReply};
use serenity::all::CreateEmbedFooter;
use tracing::error;

#[poise::command(prefix_command, track_edits, category = "Utilities")]
pub async fn compile(
    ctx: Context<'_>,
    #[rest]
    #[description = "the code to run"]
    code: String,
) -> Result<(), Error> {
    let Some((_, lang, code)) =
        regex_captures!("```(?<lang>[^\n]*)\n(?<code>.+?)\n```"sm, code.as_str())
    else {
        return Err("Cannot parse the code".into());
    };

    let duration = std::time::Instant::now();

    let client = ClientBuilder::new()
        .set_lang(lang)
        .set_main_file(code)
        .user_agent("@romancitodev/fewwis-bot")
        .build()
        .unwrap();

    let Ok(response) = client.execute().await else {
        error!("IMPORTANT ERROR HERE");
        return Err("error executing the code...".into());
    };

    let data = response.data();

    let embed = CreateEmbed::new()
        .title(format!(
            "👨‍🍳 Your code is cooked! {} ({})",
            response.language(),
            response.version()
        ))
        .color(if data.code() == 0 {
            Colors::Fewwis
        } else {
            Colors::Red
        })
        .description(
            CodeBlock {
                code: data.output().to_string(),
                ..Default::default()
            }
            .to_string(),
        )
        .footer(CreateEmbedFooter::new(format!(
            "elapsed: {}ms",
            duration.elapsed().as_millis(),
        )));

    ctx.send(CreateReply::default().embed(embed)).await.unwrap();

    Ok(())
}
