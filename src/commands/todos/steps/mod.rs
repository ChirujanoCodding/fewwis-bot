use crate::{
    helper::db::{get_all_steps, get_post},
    types::{Context, Error},
};

use crate::serenity::AutocompleteChoice;

mod add;
mod delete;
mod update;

async fn task_autocompleter(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = AutocompleteChoice> {
    let db = &ctx.data().db;
    let post = get_post(db, ctx.channel_id().get()).await.unwrap();

    let choices = get_all_steps(db, post.id).await.unwrap();

    choices
        .iter()
        .filter(|c| c.description.contains(partial))
        .map(|choice| {
            let choice = choice.clone();
            let description = format!(
                "{} - {}",
                if choice.completed != 0 { "✅" } else { "⏳" },
                choice.description
            );
            AutocompleteChoice::new(description, choice.id)
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// Set the milestone of a task
#[poise::command(
    slash_command,
    name_localized("es-ES", "pasos"),
    description_localized("es-ES", "Establece los pasos a seguir dentro de la tarea"),
    category = "Utilities",
    subcommands("add::add", "update::update", "delete::delete")
)]
pub async fn steps(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}
