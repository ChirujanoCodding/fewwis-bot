use std::fmt::Write;

use crate::consts::{TODO_CHANNEL, TODO_TAG};
use crate::helper::db::{add_steps, save_post};
use crate::helper::Colors;
use crate::types::ApplicationContext;
use crate::{Context, Error};

use ::serenity::{
    builder::{Builder, CreateAllowedMentions, CreateEmbed, CreateForumPost, CreateMessage},
    model::Color,
};
use poise::{serenity_prelude as serenity, CreateReply, Modal};

/// Shortcut to create a to-do task
#[poise::command(context_menu_command = "Create todo", category = "Utilities")]
pub async fn create_ctx_menu(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    let content = msg.content.clone();
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();
    let post = CreateForumPost::new(
        content,
        CreateMessage::new()
            .content(format!("created by {}", ctx.author().clone()))
            .allowed_mentions(CreateAllowedMentions::default().empty_users()),
    )
    .add_applied_tag(TODO_TAG.into())
    .execute(ctx, TODO_CHANNEL.into())
    .await?;

    ctx.send(
        reply
            .embed(
                embed
                    .color(Color::BLURPLE)
                    .title("✅ To-do created!")
                    .description(format!("> Just go to {}", post)),
            )
            .ephemeral(true),
    )
    .await?;
    msg.delete(ctx).await?;
    Ok(())
}

#[derive(Debug, Modal)]
#[name = "To-do List"]
struct PostModal {
    #[name = "Title of the task"]
    #[placeholder = "Buy bananas"]
    #[min_length = 1]
    title: String,
    #[name = "Description of the task"]
    #[placeholder = "Must be yellow"]
    description: String,
    #[paragraph]
    #[min_length = 0]
    list: String,
}

/// Create a to-do task
#[poise::command(slash_command, category = "Utilities")]
pub async fn create(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = PostModal::execute(ctx).await?;
    let Some(data) = data else {
        return Err("Something went wrong".into());
    };
    let db = &ctx.data().db;
    let tasks = if data.list.trim().is_empty() {
        "> _None provided_ ".to_owned()
    } else {
        format_steps(&data.list)
    };
    let reply = CreateReply::default();
    let embed = CreateEmbed::new();
    let post = CreateForumPost::new(
        data.title.clone(),
        CreateMessage::new()
            .add_embeds(vec![
                CreateEmbed::new()
                    .title(data.title.clone())
                    .description(data.description.clone())
                    .color(Colors::Fewwis),
                CreateEmbed::new()
                    .title("Tasks")
                    .description(tasks)
                    .color(Colors::Fewwis),
            ])
            .allowed_mentions(CreateAllowedMentions::default().empty_users()),
    )
    .add_applied_tag(TODO_TAG.into())
    .execute(ctx.http(), TODO_CHANNEL.into())
    .await?;

    ctx.send(
        reply
            .embed(
                embed
                    .color(Color::BLURPLE)
                    .title("✅ To-do created!")
                    .description(format!("> Just go to {}", post)),
            )
            .ephemeral(true),
    )
    .await?;

    {
        let post = save_post(db, post.id.get(), data.title).await?;

        if !data.list.trim().is_empty() {
            add_steps(
                db,
                post.id,
                data.list.clone().split('\n').map(String::from).collect(),
            )
            .await?;
        }
    };
    Ok(())
}

fn format_steps(steps: &str) -> String {
    steps
        .split('\n')
        .enumerate()
        .fold(String::new(), |mut acc, (index, task)| {
            writeln!(&mut acc, "> ⏳ **{}.** {}", index + 1, task).unwrap();
            acc
        })
}
