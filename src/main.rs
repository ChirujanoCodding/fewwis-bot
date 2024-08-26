use ::serenity::gateway::ActivityData;
use consts::OWNER_BOT;
use helper::handle_error;
use poise::serenity_prelude as serenity;
use sea_orm::{ConnectionTrait, Database, Statement};
use std::{collections::HashSet, sync::Arc};
use tracing::{error, info};
mod api;
mod commands;
mod consts;
mod entities;
mod helper;
mod types;
use types::*;

#[tokio::main]
async fn main() {
    if let Err(e) = dotenvy::dotenv() {
        error!("❌ `dotenv`: {e}");
        std::process::exit(0);
    };

    let token = dotenvy::var("BOT_TOKEN").expect("❌ Missing BOT_TOKEN in .env file");

    let intents = serenity::GatewayIntents::all();

    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            owners: HashSet::from([OWNER_BOT.into()]),
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "📥 Starting interaction (command {})",
                        &ctx.invoked_command_name()
                    );
                })
            },
            post_command: |ctx| {
                Box::pin(
                    async move { info!("✅ Command executed ({})", &ctx.invoked_command_name()) },
                )
            },
            on_error: |error| Box::pin(handle_error(error)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, fm| {
            Box::pin(async move {
                info!("👷 Setting up the bot...");
                ctx.set_activity(Some(ActivityData::playing("Casio Theme 4 life")));
                info!("🕹  Setted activity.");
                let commands = &fm.options().commands;
                let create_commands = poise::builtins::create_application_commands(commands);

                info!("🔁 Registering commands...");
                serenity::Command::set_global_commands(ctx, create_commands).await?;
                info!("📤 Registered commands.");
                info!("📡 Connecting to database...");

                let db_url = dotenvy::var("DATABASE_URL")?;
                let db_name = dotenvy::var("DATABASE_NAME")?;

                let connection = Database::connect(db_url.clone()).await?;
                connection
                    .execute(Statement::from_string(
                        connection.get_database_backend(),
                        format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name.clone()),
                    ))
                    .await?;
                let connection = Database::connect(format!("{db_url}{db_name}")).await?;
                info!("📡 Connection to database successfull.");
                info!("✅ Bot initialized.");
                Ok(Data::new(connection))
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}
