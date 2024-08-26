use raelib::prelude::RaeClient;
use rand::{self, seq::SliceRandom};
use sea_orm::EntityTrait;
use std::time::Duration;

use crate::{
    entities::buttons,
    helper::{
        db::{get_buttons, get_user},
        Colors,
    },
    Context, Error,
};
use ::serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model,
};
use poise::{serenity_prelude as serenity, CreateReply};

#[derive(PartialEq, Eq)]
enum Reason {
    Correct,
    Loss,
    Timeout,
}

/// Set the quiz about words
#[poise::command(
    slash_command,
    name_localized("es-ES", "definiciones"),
    description_localized("es-ES", "Trivia de definiciones de palabras!"),
    subcommands("play_buttons", "stats_buttons"),
    category = "Games"
)]
pub async fn definitions(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Play a quiz of words!
#[poise::command(
    slash_command,
    name_localized("es-ES", "jugar"),
    description_localized("es-ES", "Juega a una trivia de definiciones del diccionario!"),
    rename = "play",
    category = "Games"
)]
pub async fn play_buttons(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let msg = ctx
        .send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("⏳ Starting game...")
                    .color(Colors::White),
            ),
        )
        .await?;
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();
    let db = &ctx.data().db;
    let (user_id, guild_id) = (ctx.author().id, ctx.guild_id().unwrap());
    let (_, stats) = get_user(user_id, guild_id, db).await?;
    let request_buttons = get_buttons(stats, db).await?;

    let rae_client = RaeClient::default();
    let (words, correct) = {
        let mut w = rae_client.get_random_with_range(0..4).await?;
        let mut rng = rand::thread_rng();
        w.shuffle(&mut rng);
        let correct = w.as_slice().choose(&mut rng).unwrap().clone();
        (w, correct)
    };

    let buttons = words
        .iter()
        .enumerate()
        .map(|(idx, word)| {
            let mut label = word.word().to_string();
            if let Some(variant) = word.variant() {
                label += &*format!(" / {}", variant);
            };
            let id = if *word == correct {
                format!("correct-{}-{}", idx, ctx.id())
            } else {
                format!("wrong-{}-{}", idx, ctx.id())
            };
            CreateButton::new(id)
                .label(label)
                .style(serenity::ButtonStyle::Secondary)
        })
        .collect::<Vec<_>>();

    let row = CreateActionRow::Buttons(buttons);

    let mut reason = Reason::Timeout;

    let definitions = correct
        .definitions()
        .iter()
        .enumerate()
        .map(|(i, def)| {
            format!(
                "**{n}.** _{dtype}_\n> {definition}\n",
                n = i + 1,
                dtype = def.def_type(),
                definition = def.definition()
            )
        })
        .collect::<Vec<_>>()
        .join("> ");

    msg.edit(
        ctx,
        reply
            .clone()
            .embed(
                embed
                    .clone()
                    .title("🤓 Quiz of Definitions")
                    .color(Colors::Fewwis)
                    .field("☝️ Definition:", format!("> {}", definitions), true),
            )
            .components(vec![row]),
    )
    .await?;

    let message_id = msg.message().await?.id;

    if let Some(interaction) =
        serenity::ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
            .author_id(ctx.author().id)
            .message_id(message_id)
            .timeout(Duration::from_secs(60))
            .await
    {
        let [is_correct, index, ..] = interaction.data.custom_id.split('-').collect::<Vec<_>>()[..]
        else {
            return Err("Error spliting id".into());
        };

        let is_correct = is_correct == "correct";
        let embed = embed
            .clone()
            .title("🤓 Quiz of Definitions")
            .description(if is_correct {
                "> 🎉 **Good job!**, you guessed the word!"
            } else {
                "> ❌ **Oops...** You failed. "
            })
            .color(if is_correct {
                Colors::Green
            } else {
                Colors::Red
            })
            .field("☝️ Definition:", format!(">  {}", definitions), true);
        reason = if is_correct {
            Reason::Correct
        } else {
            Reason::Loss
        };
        let buttons = words
            .iter()
            .enumerate()
            .map(|(idx, word)| {
                let mut label = word.word().to_string();
                if let Some(variant) = word.variant() {
                    label += &*format!(" / {}", variant);
                };
                let style = if *word == correct {
                    serenity::ButtonStyle::Success
                } else if !is_correct && (idx == index.parse::<usize>().unwrap()) {
                    serenity::ButtonStyle::Danger
                } else {
                    serenity::ButtonStyle::Secondary
                };
                CreateButton::new(format!("answered-{}-{}", idx, ctx.id()))
                    .label(label)
                    .style(style)
                    .disabled(true)
            })
            .collect::<Vec<_>>();
        let row = CreateActionRow::Buttons(buttons);
        interaction
            .create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
            .await?;
        msg.edit(ctx, reply.clone().embed(embed).components(vec![row]))
            .await?;
    }

    if reason == Reason::Timeout {
        let embed = embed
            .clone()
            .title("🤓 Quiz of Definitions")
            .description("> ⌛ **Oops...** Timeout. ")
            .color(Colors::Red)
            .field("☝️ Definition:", format!("> {}", definitions), true);
        let buttons = words
            .iter()
            .enumerate()
            .map(|(idx, word)| {
                let mut label = word.word().to_string();
                if let Some(variant) = word.variant() {
                    label += &*format!(" / {}", variant);
                };
                let style = if *word == correct {
                    serenity::ButtonStyle::Success
                } else {
                    serenity::ButtonStyle::Secondary
                };
                CreateButton::new(format!("answered-{}-{}", idx, ctx.id()))
                    .label(label)
                    .style(style)
                    .disabled(true)
            })
            .collect::<Vec<_>>();
        let row = CreateActionRow::Buttons(buttons);
        msg.edit(ctx, reply.clone().embed(embed).components(vec![row]))
            .await?
    }

    let active_buttons: buttons::ActiveModel = request_buttons.into();

    let updated_buttons = match reason {
        Reason::Correct => buttons::ActiveModel {
            asserted: sea_orm::ActiveValue::Set(active_buttons.asserted.unwrap() + 1),
            ..active_buttons
        },
        Reason::Loss | Reason::Timeout => buttons::ActiveModel {
            wrong: sea_orm::ActiveValue::Set(active_buttons.wrong.unwrap() + 1),
            ..active_buttons
        },
    };

    buttons::Entity::update(updated_buttons).exec(db).await?;

    Ok(())
}

/// View your stats in quiz words
#[poise::command(
    slash_command,
    name_localized("es-ES", "estadisticas"),
    description_localized("es-ES", "Mira tus estadisticas sobre la trivia de definiciones!"),
    rename = "stats",
    category = "Games"
)]
pub async fn stats_buttons(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let msg = ctx
        .send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("⏳ Fetching data...")
                    .color(Colors::White),
            ),
        )
        .await?;
    let db = &ctx.data().db;
    let (user_id, guild_id) = (ctx.author().id, ctx.guild_id().unwrap());
    let (_, stats) = get_user(user_id, guild_id, db).await?;
    let buttons = get_buttons(stats, db).await?;

    let total = buttons.asserted + buttons.wrong;

    let asserted_percentage = if total == 0 {
        0.0
    } else {
        (buttons.asserted as f64 / total as f64) * 100.0
    };

    let wrong_percentage = if total == 0 {
        0.0
    } else {
        (buttons.wrong as f64 / total as f64) * 100.0
    };

    msg.edit(
        ctx,
        CreateReply::default().embed(
            CreateEmbed::new()
                .title(format!("🏁 Quiz stats of {}", ctx.author().name))
                .color(model::Color::BLURPLE)
                .fields([
                    (
                        "✅ Asserted:",
                        format!("{} ({:.2}%)", buttons.asserted, asserted_percentage),
                        true,
                    ),
                    (
                        "❌ Wrong:",
                        format!("{} ({:.2}%)", buttons.wrong, wrong_percentage),
                        true,
                    ),
                    ("🔢 Total:", format!("{}", total), true),
                ]),
        ),
    )
    .await?;

    Ok(())
}
