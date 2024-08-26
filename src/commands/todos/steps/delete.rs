use std::fmt::Write;

use crate::{
    helper::{
        db::{delete_task, get_all_steps, get_post},
        Colors,
    },
    types::{Context, Error},
};
use serenity::builder::{CreateEmbed, EditMessage};

/// Update the state of a step.
#[poise::command(
    slash_command,
    name_localized("es-ES", "eliminar"),
    description_localized("es-ES", "Elimina un paso"),
    category = "Utilities"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The task to eliminar"]
    #[name_localized("es-ES", "tarea")]
    #[description_localized("es-ES", "La tarea a eliminar")]
    #[autocomplete = "super::task_autocompleter"]
    task: i32, // The autocompleter just return the value of itself.
) -> Result<(), Error> {
    let db = &ctx.data().db;

    let post = get_post(db, ctx.channel_id().get()).await?;
    delete_task(db, task).await?;

    let tasks = get_all_steps(db, post.id)
        .await?
        .iter()
        .fold(String::new(), |mut acc, task| {
            writeln!(
                &mut acc,
                "> {} **{}.** {}",
                if task.completed != 0 { "✅" } else { "⏳" },
                task.index,
                task.description
            )
            .unwrap();
            acc
        });

    let message = ctx.channel_id().message(ctx.http(), post.post_id).await?;

    let mut embeds = message
        .embeds
        .iter()
        .cloned()
        .map(CreateEmbed::from)
        .collect::<Vec<_>>();

    embeds[1] = CreateEmbed::new()
        .title("Tasks")
        .description(if tasks.is_empty() { "> _None_" } else { &tasks })
        .color(Colors::Fewwis);

    ctx.channel_id()
        .edit_message(
            ctx.http(),
            message.id,
            EditMessage::new().add_embeds(embeds),
        )
        .await?;

    ctx.defer_ephemeral().await?;
    ctx.say("Task deleted!").await?;

    Ok(())
}
