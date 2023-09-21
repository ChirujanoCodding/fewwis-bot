use rand::{self, seq::SliceRandom, Rng};
use sea_orm::EntityTrait;
use std::time::Duration;

use crate::{
    api::{Countries, FLAGS_API},
    entities::flags,
    helper::{
        db::{get_flags, get_user},
        Colors,
    },
    Context, Error,
};
use ::serenity::{
    builder::{CreateEmbed, CreateEmbedFooter},
    model,
};
use poise::{serenity_prelude as serenity, CreateReply};

enum Reason {
    Correct,
    Loss,
    Timeout,
}

/// Play a quiz!
#[poise::command(
    slash_command,
    name_localized("es-ES", "trivia"),
    description_localized("es-ES", "Juega a una trivia!"),
    subcommands("flags"),
    category = "Games"
)]
pub async fn quiz(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the quiz about flags
#[poise::command(
    slash_command,
    name_localized("es-ES", "banderas"),
    description_localized("es-ES", "Trivia de banderas!"),
    subcommands("play_flags", "stats_flags"),
    category = "Games"
)]
pub async fn flags(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Play a quiz of flags!
#[poise::command(
    slash_command,
    name_localized("es-ES", "jugar"),
    description_localized("es-ES", "Juega a una trivia de banderas!"),
    rename = "play",
    category = "Games"
)]
pub async fn play_flags(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let msg = ctx
        .send(
            CreateReply::new().embed(
                CreateEmbed::new()
                    .title("⏳ Starting game...")
                    .color(Colors::White),
            ),
        )
        .await?;
    const MAX_RETRIES: i32 = 3;
    let reply = CreateReply::new();
    let embed = CreateEmbed::new();
    let db = &ctx.data().db;
    let (user_id, guild_id) = (ctx.author().id, ctx.guild_id().unwrap());
    let (_, stats) = get_user(user_id, guild_id, db).await?;
    let request_flags = get_flags(stats, db).await?;

    let mut request = reqwest::get(FLAGS_API).await?.json::<Countries>().await?;
    let flag = {
        let mut rng = rand::thread_rng();
        request.shuffle(&mut rng);
        request[rng.gen_range(0..request.len() - 1)].clone()
    };

    let mut counter = 0;
    let mut reason = Reason::Timeout;

    msg.edit(
        ctx,
        reply.clone().embed(
            embed
                .clone()
                .title("🏁 Quiz of Flags")
                .color(Colors::Fewwis)
                .image(&flag.flags.png)
                .footer(CreateEmbedFooter::new(format!(
                    "Tries: {}/{}",
                    counter + 1,
                    MAX_RETRIES
                ))),
        ),
    )
    .await?;

    while let Some(message) = serenity::MessageCollector::new(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(60))
        .await
    {
        let flag = flag.clone();

        if ![
            flag.name.common.to_lowercase(),
            flag.name.official.to_lowercase(),
            flag.translations.spa.common.to_lowercase(),
            flag.translations.spa.official.to_lowercase(),
        ]
        .contains(&message.content.to_lowercase())
        {
            message.react(ctx, '❌').await?;
            counter += 1;
        } else {
            message.react(ctx, '✅').await?;
            reason = Reason::Correct;
            break;
        }

        let mut embed = embed
            .clone()
            .title("🏁 Quiz of Flags")
            .color(Colors::Fewwis)
            .image(&flag.flags.png)
            .footer(CreateEmbedFooter::new(format!(
                "Tries: {}/{}",
                counter + 1,
                MAX_RETRIES
            )));

        if counter >= MAX_RETRIES {
            msg.edit(
                ctx,
                reply.clone().embed(
                    embed
                        .clone()
                        .thumbnail(flag.coat_of_arms.png.clone())
                        .description("> 🤓:point_up: The name of the country is:")
                        .fields([
                            (
                                ":flag_us: English",
                                format!("{}/{}", flag.name.common, flag.name.official),
                                true,
                            ),
                            (
                                ":flag_es: Spanish",
                                format!(
                                    "{}/{}",
                                    flag.translations.spa.common, flag.translations.spa.official
                                ),
                                true,
                            ),
                        ]),
                ),
            )
            .await?;
            reason = Reason::Loss;
            break;
        };

        if counter == MAX_RETRIES - 1 {
            embed = embed
                .thumbnail(flag.coat_of_arms.png)
                .description("> 💡 **HINT:** Look the thumbnail.");
        }
        msg.edit(ctx, reply.clone().embed(embed)).await?;
    }

    let active_flags: flags::ActiveModel = request_flags.into();
    let updated_flags = match reason {
        Reason::Correct => {
            msg.edit(
                ctx,
                reply.clone().embed(
                    embed
                        .color(Colors::Green)
                        .description("> 🎉 **Congrats!** You got it right."),
                ),
            )
            .await?;
            match counter {
                0 => flags::ActiveModel {
                    first_attempt: sea_orm::ActiveValue::Set(
                        active_flags.first_attempt.unwrap() + 1,
                    ),
                    ..active_flags
                },
                1 => flags::ActiveModel {
                    second_attempt: sea_orm::ActiveValue::Set(
                        active_flags.second_attempt.unwrap() + 1,
                    ),
                    ..active_flags
                },
                2 => flags::ActiveModel {
                    third_attempt: sea_orm::ActiveValue::Set(
                        active_flags.third_attempt.unwrap() + 1,
                    ),
                    ..active_flags
                },
                _ => unreachable!(),
            }
        }
        Reason::Loss => flags::ActiveModel {
            wrong: sea_orm::ActiveValue::Set(active_flags.wrong.unwrap() + 1),
            ..active_flags
        },
        Reason::Timeout => active_flags,
    };

    flags::Entity::update(updated_flags).exec(db).await?;

    Ok(())
}

/// View your stats in quiz flags
#[poise::command(
    slash_command,
    name_localized("es-ES", "estadisticas"),
    description_localized("es-ES", "Mira tus estadisticas sobre la trivia de banderas!"),
    rename = "stats",
    category = "Games"
)]
pub async fn stats_flags(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let msg = ctx
        .send(
            CreateReply::new().embed(
                CreateEmbed::new()
                    .title("⏳ Fetching data...")
                    .color(Colors::White),
            ),
        )
        .await?;
    let db = &ctx.data().db;
    let (user_id, guild_id) = (ctx.author().id, ctx.guild_id().unwrap());
    let (_, stats) = get_user(user_id, guild_id, db).await?;
    let flags = get_flags(stats, db).await?;

    let total = flags.first_attempt + flags.second_attempt + flags.third_attempt + flags.wrong;

    let first_attempt_percentage = if total == 0 {
        0.0
    } else {
        (flags.first_attempt as f64 / total as f64) * 100.0
    };
    let second_attempt_percentage = if total == 0 {
        0.0
    } else {
        (flags.second_attempt as f64 / total as f64) * 100.0
    };
    let third_attempt_percentage = if total == 0 {
        0.0
    } else {
        (flags.third_attempt as f64 / total as f64) * 100.0
    };
    let wrong_percentage = if total == 0 {
        0.0
    } else {
        (flags.wrong as f64 / total as f64) * 100.0
    };

    msg.edit(
        ctx,
        CreateReply::new().embed(
            CreateEmbed::new()
                .title(format!("🏁 Quiz stats of {}", ctx.author().name))
                .color(model::Color::BLURPLE)
                .fields([
                    (
                        "1️⃣ Attempt:",
                        format!("{} ({:.2}%)", flags.first_attempt, first_attempt_percentage),
                        true,
                    ),
                    (
                        "2️⃣ Attempt:",
                        format!(
                            "{} ({:.2}%)",
                            flags.second_attempt, second_attempt_percentage
                        ),
                        true,
                    ),
                    (
                        "3️⃣ Attempt:",
                        format!("{} ({:.2}%)", flags.third_attempt, third_attempt_percentage),
                        true,
                    ),
                    (
                        "❌ Wrong:",
                        format!("{} ({:.2}%)", flags.wrong, wrong_percentage),
                        true,
                    ),
                    ("🔢 Total:", format!("{}", total), true),
                ]),
        ),
    )
    .await?;

    Ok(())
}
