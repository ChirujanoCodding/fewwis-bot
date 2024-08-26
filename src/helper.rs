use poise::{serenity_prelude as serenity, serenity_prelude::Color, CreateReply};
use serenity::builder::{CreateActionRow, CreateButton, CreateEmbed};
use std::time::Duration;
use tracing::error;

use crate::types::{Context, Data, Error};

pub enum Colors {
    White,
    Gray,
    Green,
    Orange,
    Red,
    Fewwis,
    Custom(u8, u8, u8),
}

impl From<Colors> for Color {
    fn from(value: Colors) -> Self {
        match value {
            Colors::White => Color::from_rgb(255, 255, 255),
            Colors::Gray => Color::from_rgb(175, 175, 175),
            Colors::Green => Color::from_rgb(178, 247, 117),
            Colors::Orange => Color::from_rgb(247, 194, 131),
            Colors::Red => Color::from_rgb(247, 131, 131),
            Colors::Fewwis => Color::from_rgb(231, 127, 34),
            Colors::Custom(r, g, b) => Color::from_rgb(r, g, b),
        }
    }
}

pub async fn handle_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            let reply = CreateReply::default();
            let embed = CreateEmbed::new();
            ctx.send(
                reply
                    .embed(
                        embed
                            .title("❌ Oops... an error ocurred.")
                            .description(error.to_string())
                            .color(serenity::model::Color::RED),
                    )
                    .ephemeral(true),
            )
            .await
            .unwrap();
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            let reply = CreateReply::default();
            let embed = CreateEmbed::new();
            ctx.send(
                reply
                    .embed(
                        embed
                            .title("❌ Oops... an error ocurred.")
                            .description(error.unwrap_or("Unexpected".into()).to_string())
                            .color(serenity::model::Color::RED),
                    )
                    .ephemeral(true),
            )
            .await
            .unwrap();
        }
        poise::FrameworkError::Setup { error, .. } => {
            error!("❌ Errot setting up the bot. {error:?}");
        }
        err => {
            if let Err(err) = poise::builtins::on_error(err).await {
                error!("❌ Error handling {err}");
            }
        }
    };
}

type Function = fn(&Context, &str, i32, i32) -> i32;
type Conditional = fn(usize, i32, i32) -> bool;

pub struct Paginator {
    pages: Vec<CreateEmbed>,
    counter: i32,
    additional_fn: Option<Function>,
    additional_cond: Option<Conditional>,
    additional_row: Option<CreateActionRow>,
}

impl Paginator {
    pub fn new(pages: Vec<CreateEmbed>) -> Paginator {
        Paginator {
            pages,
            counter: 0,
            additional_fn: None,
            additional_cond: None,
            additional_row: None,
        }
    }

    pub fn add_row(self, row: CreateActionRow, r#fn: Function, cond: Conditional) -> Self {
        Self {
            additional_row: Some(row),
            additional_fn: Some(r#fn),
            additional_cond: Some(cond),
            ..self
        }
    }

    pub async fn paginate(&mut self, ctx: Context<'_>) -> Result<(), Error> {
        let reply = CreateReply::default();
        let buttons = self.create_buttons();

        let initial = ctx
            .send(
                reply
                    .clone()
                    .embed(self.pages[self.counter as usize].clone())
                    .components(buttons),
            )
            .await?;

        let message_id = initial.message().await?.id;

        while let Some(interaction) = initial
            .message()
            .await?
            .await_component_interactions(ctx.serenity_context().shard.clone())
            .message_id(message_id)
            .timeout(Duration::from_secs(60))
            .await
        {
            match &*interaction.data.custom_id {
                "left" => {
                    self.counter = 0.max(self.counter - 1);
                }
                "right" => {
                    self.counter = (self.pages.len() as i32 - 1).min(self.counter + 1);
                }
                "beggining" => self.counter = 0,
                "final" => {
                    self.counter = self.pages.len() as i32 - 1;
                }
                "delete" => {
                    initial.delete(ctx).await?;
                    return Ok(());
                }
                id => {
                    if let Some(additional_fn) = &self.additional_fn {
                        self.counter =
                            (additional_fn)(&ctx, id, self.counter, self.pages.len() as i32 - 1);
                    }
                }
            };

            let buttons = self.create_buttons();

            interaction
                .create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
                .await?;
            initial
                .edit(
                    ctx,
                    reply
                        .clone()
                        .embed(self.pages[self.counter as usize].clone())
                        .components(buttons),
                )
                .await?;
        }

        initial
            .edit(
                ctx,
                reply
                    .clone()
                    .embed(self.pages[self.counter as usize].clone())
                    .components(vec![]),
            )
            .await?;
        Ok(())
    }

    pub(self) fn create_buttons(&self) -> Vec<CreateActionRow> {
        let mut buttons_row = vec![];
        let left = CreateButton::new("left")
            .style(serenity::ButtonStyle::Primary)
            .label("◀")
            .disabled(self.counter == 0);
        let center = CreateButton::new("center")
            .label(format!("{}/{}", self.counter + 1, self.pages.len()))
            .disabled(true)
            .style(serenity::ButtonStyle::Secondary);
        let right = CreateButton::new("right")
            .style(serenity::ButtonStyle::Primary)
            .label("▶")
            .disabled(self.counter >= self.pages.len() as i32 - 1);
        let to_beggining = CreateButton::new("beggining")
            .style(serenity::ButtonStyle::Primary)
            .label("⏪")
            .disabled(self.counter == 0);
        let to_final = CreateButton::new("final")
            .style(serenity::ButtonStyle::Primary)
            .label("⏩")
            .disabled(self.counter >= self.pages.len() as i32 - 1);

        let buttons = CreateActionRow::Buttons(vec![to_beggining, left, center, right, to_final]);

        buttons_row.push(buttons);

        if let Some(CreateActionRow::Buttons(additional_rows)) = &self.additional_row {
            let rows = CreateActionRow::Buttons(
                additional_rows
                    .iter()
                    .cloned()
                    .enumerate()
                    .map({
                        |(index, b)| {
                            let is_disabled = self.additional_cond.unwrap()(
                                index + 1,
                                self.counter,
                                self.pages.len() as i32,
                            );
                            b.disabled(is_disabled)
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            buttons_row.push(rows)
        }

        buttons_row
    }
}

pub mod db {

    use crate::entities::flags::{self, Entity as Flags};
    use crate::entities::rel_users_stats::{self, Entity as Relation};
    use crate::entities::stats::{self, Entity as Stats};
    use crate::entities::users::{self, Entity as Users};
    use crate::entities::{buttons, rel_buttons_stats, rel_flags_stats, step, task};
    use crate::types::Error;
    use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
    use serenity::all::{GuildId, UserId};
    pub async fn get_user(
        user_id: UserId,
        guild_id: GuildId,
        db: &DatabaseConnection,
    ) -> Result<(users::Model, stats::Model), Error> {
        let user = Users::find()
            .filter(users::Column::User.eq(user_id.get()))
            .filter(users::Column::Guild.eq(guild_id.get()))
            .one(db)
            .await?;
        if let Some(user) = user {
            let relation = Relation::find()
                .filter(rel_users_stats::Column::UsersId.eq(user.id))
                .one(db)
                .await?
                .unwrap()
                .stats_id;
            let stats = Stats::find_by_id(relation).one(db).await?.unwrap();
            Ok((user, stats))
        } else {
            let user = Users::insert(users::ActiveModel {
                user: ActiveValue::Set(user_id.get()),
                guild: ActiveValue::Set(guild_id.get()),
                ..Default::default()
            })
            .exec_with_returning(db)
            .await?;
            let stats = Stats::insert(stats::ActiveModel::default())
                .exec_with_returning(db)
                .await?;
            Relation::insert(rel_users_stats::ActiveModel {
                stats_id: ActiveValue::Set(stats.id),
                users_id: ActiveValue::Set(user.id),
                ..Default::default()
            })
            .exec(db)
            .await?;
            Ok((user, stats))
        }
    }

    pub async fn get_flags(
        stats_id: stats::Model,
        db: &DatabaseConnection,
    ) -> Result<flags::Model, Error> {
        use rel_flags_stats::Entity as Relation;
        let relation = Relation::find()
            .filter(rel_flags_stats::Column::StatsId.eq(stats_id.id))
            .one(db)
            .await?;

        if let Some(flags) = relation {
            let flags = Flags::find_by_id(flags.flags_id).one(db).await?.unwrap();
            Ok(flags)
        } else {
            let flags = Flags::insert(flags::ActiveModel {
                first_attempt: ActiveValue::Set(0),
                second_attempt: ActiveValue::Set(0),
                third_attempt: ActiveValue::Set(0),
                wrong: ActiveValue::Set(0),
                ..Default::default()
            })
            .exec_with_returning(db)
            .await?;

            Relation::insert(rel_flags_stats::ActiveModel {
                stats_id: ActiveValue::Set(stats_id.id),
                flags_id: ActiveValue::Set(flags.id),
                ..Default::default()
            })
            .exec(db)
            .await?;

            Ok(flags)
        }
    }

    pub async fn get_buttons(
        stats_id: stats::Model,
        db: &DatabaseConnection,
    ) -> Result<buttons::Model, Error> {
        use rel_buttons_stats::Entity as Relation;
        let relation = Relation::find()
            .filter(rel_buttons_stats::Column::StatsId.eq(stats_id.id))
            .one(db)
            .await?;

        if let Some(buttons) = relation {
            let buttons = buttons::Entity::find_by_id(buttons.buttons_id)
                .one(db)
                .await?
                .unwrap();
            Ok(buttons)
        } else {
            let buttons = buttons::Entity::insert(buttons::ActiveModel {
                asserted: ActiveValue::Set(0),
                wrong: ActiveValue::Set(0),
                ..Default::default()
            })
            .exec_with_returning(db)
            .await?;

            Relation::insert(rel_buttons_stats::ActiveModel {
                stats_id: ActiveValue::Set(stats_id.id),
                buttons_id: ActiveValue::Set(buttons.id),
                ..Default::default()
            })
            .exec(db)
            .await?;

            Ok(buttons)
        }
    }
    pub async fn get_post(db: &DatabaseConnection, post_id: u64) -> Result<task::Model, Error> {
        use task::Entity as Task;
        let tasks = Task::find()
            .filter(task::Column::PostId.eq(post_id))
            .one(db)
            .await?;

        if let Some(tasks) = tasks {
            return Ok(tasks);
        }
        Err("Cannot find task channel...".into())
    }

    pub async fn save_post(
        db: &DatabaseConnection,
        post_id: u64,
        title: String,
    ) -> Result<task::Model, Error> {
        use task::{ActiveModel, Entity as Task};
        Task::insert(ActiveModel {
            post_id: sea_orm::ActiveValue::Set(post_id),
            title: sea_orm::ActiveValue::Set(title),
            ..Default::default()
        })
        .exec_with_returning(db)
        .await
        .map_err(|x| x.into())
    }

    pub async fn add_steps(
        db: &DatabaseConnection,
        task_id: i32,
        steps: Vec<String>,
    ) -> Result<(), Error> {
        use step::Entity as Step;
        let last_id = get_last_index(db, task_id).await.unwrap_or_default();

        for (i, step) in steps.iter().enumerate() {
            Step::insert(step::ActiveModel {
                index: ActiveValue::Set(last_id + i as i64 + 1),
                description: ActiveValue::Set(step.clone()),
                task_id: ActiveValue::Set(task_id),
                completed: ActiveValue::Set(0),
                ..Default::default()
            })
            .exec(db)
            .await?;
        }

        Ok(())
    }

    async fn get_last_index(db: &DatabaseConnection, task_id: i32) -> Option<i64> {
        use step::Entity as Step;
        let vec = &Step::find()
            .filter(step::Column::TaskId.eq(task_id))
            .all(db)
            .await
            .unwrap();
        vec.last().map(|model| model.index)
    }

    pub async fn get_all_steps(
        db: &DatabaseConnection,
        task_id: i32,
    ) -> Result<Vec<step::Model>, Error> {
        use step::Entity as Step;
        let vec = Step::find()
            .filter(step::Column::TaskId.eq(task_id))
            .all(db)
            .await?;

        Ok(vec)
    }

    pub async fn delete_post(db: &DatabaseConnection, post_id: u64) -> Result<(), Error> {
        use task::Entity as Task;
        let model = Task::find()
            .filter(task::Column::PostId.eq(post_id))
            .one(db)
            .await?;
        if let Some(model) = model {
            Task::delete(task::ActiveModel {
                id: ActiveValue::Set(model.id),
                ..Default::default()
            })
            .exec(db)
            .await?;
            Ok(())
        } else {
            Err("❌ Something went wrong...".into())
        }
    }

    pub async fn update_task(
        db: &DatabaseConnection,
        step_id: i32,
        state: bool,
    ) -> Result<(), Error> {
        use step::Entity as Step;

        let model = step::ActiveModel {
            id: ActiveValue::Set(step_id),
            completed: ActiveValue::Set(state as i8),
            ..Default::default()
        };

        Step::update(model).exec(db).await?;
        Ok(())
    }

    pub async fn delete_task(db: &DatabaseConnection, step_id: i32) -> Result<(), Error> {
        use step::Entity as Step;

        let model = step::ActiveModel {
            id: ActiveValue::Set(step_id),
            ..Default::default()
        };
        Step::delete(model).exec(db).await?;

        Ok(())
    }
}
