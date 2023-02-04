pub mod commands;
pub mod model;
pub mod schema;

use commands::log_letters::log_letter;
use commands::publish;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

use dotenv::dotenv;
use serenity::futures::TryFutureExt;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::CommandId;
use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler {
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                // "ping" => commands::ping::run(&command.data.options),
                "sendletter" => {
                    match commands::send::run(
                        &command.data.options,
                        &command.user,
                        &mut self.db_pool.get().unwrap(),
                    ) {
                        Ok((letter, message)) => {
                            log_letter(&ctx, &letter).await;
                            message
                        }
                        Err(message) => message,
                    }
                }
                "publish" => publish::run(&command, &ctx, &mut self.db_pool.get().unwrap())
                    .await
                    .map_or_else(|e| e, |m| m),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(content).ephemeral(true)
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // let guild_id = GuildId(
        //     env::var("GUILD_ID")
        //         .expect("Expected GUILD_ID in environment")
        //         .parse()
        //         .expect("GUILD_ID must be an integer"),
        // );

        // guild_id
        //     .delete_application_command(&ctx.http, CommandId(1070794475665891348))
        //     .unwrap_or_else(|f| {
        //         println!("Shit's fucked {}", f.to_string());
        //     })
        //     .await;

        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
            .create_application_command(|command| commands::send::register(command))
            .create_application_command(|command| commands::publish::register(command))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    run_migrations(&mut SqliteConnection::establish(&database_url).unwrap()).unwrap();

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler {
            db_pool: Pool::builder()
                .test_on_check_out(true)
                .build(ConnectionManager::<SqliteConnection>::new(database_url))
                .expect("Could not build connection pool"),
        })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
