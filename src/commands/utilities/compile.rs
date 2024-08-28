use crate::{helper::Colors, Context, Error};
use pistones::Client;

use ::serenity::builder::CreateEmbed;
use poise::{CodeBlock, CreateReply};
use serenity::all::CreateEmbedFooter;

#[poise::command(prefix_command, track_edits, category = "Utilities")]
pub async fn compile(
    ctx: Context<'_>,
    #[rest]
    #[description = "the code to run"]
    code: String,
) -> Result<(), Error> {
    let reply = CreateReply::default();

    let Ok((lang, code)) = parse_code(&code) else {
        return Err("Error parsing the code.".into());
    };

    let Ok(data) = compile_code(lang, code).await else {
        return Err("Cannot compile the code dud".into());
    };

    let embed = create_result_embed(data);

    ctx.send(reply.embed(embed)).await.unwrap();

    Ok(())
}

async fn compile_code(
    lang: &str,
    code: &str,
) -> Result<pistones::lang::Response, pistones::error::Error> {
    let client = Client::new().await?.user_agent("@romancitodev")?;
    client.run(lang, code).await
}

fn parse_code(code: &str) -> Result<(&str, &str), Error> {
    lazy_regex::regex_captures!("```(?<lang>[^\n]*)\n(?<code>.+?)\n```"sm, code)
        .map_or(Err("Error parsing the code".into()), |(_, lang, code)| {
            Ok((lang, code))
        })
}

fn create_result_embed(response: pistones::lang::Response) -> CreateEmbed {
    let data = response.data();
    let success = data.code() == 0;
    let (emoji, color) = if success {
        ("👩‍🍳", Colors::Fewwis)
    } else {
        ("💀", Colors::Red)
    };

    let embed = CreateEmbed::new()
        .title(format!(
            "{} Your code is cooked! {} ({})",
            emoji,
            response.language(),
            response.version()
        ))
        .color(color);

    let description = CodeBlock {
        code: data.output().to_owned(),
        ..Default::default()
    };

    if success {
        embed
            .description(description.to_string())
            .footer(CreateEmbedFooter::new("compiled successfully"))
    } else {
        embed.description(format!("Error: {}", response.data().output()))
    }
}
