pub mod commands;
pub mod model;
pub mod schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::{Sqlite, SqliteConnection};
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
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::prelude::*;

pub struct Handler {
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
    letters_allowed: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        #![allow(clippy::single_match)]
        match interaction {
            Interaction::ApplicationCommand(command) => {
                // println!("Received command interaction: {:#?}", command);

                use commands::{add_recipient, allow_letters, publish, send};

                let result = match command.data.name.as_str() {
                    // "ping" => commands::ping::run(&command.data.options),
                    "sendletter" if self.letters_allowed.load(Ordering::SeqCst) => {
                        send::run(&command, &ctx, &mut self.db_pool.get().unwrap()).await
                    }
                    "sendletter" if !self.letters_allowed.load(Ordering::SeqCst) => {
                        send::forbidden().await
                    }
                    "publish" => {
                        publish::run(&command, &ctx, &mut self.db_pool.get().unwrap()).await
                    }
                    "add_recipient" => {
                        add_recipient::run(&command, &ctx, &mut self.db_pool.get().unwrap()).await
                    }
                    "allow_letters" => {
                        allow_letters::run(&command, &ctx, &mut self.db_pool.get().unwrap())
                            .await
                            .map(|opt| {
                                opt.map(|(allowed, ret)| {
                                    self.letters_allowed.store(allowed, Ordering::SeqCst);
                                    ret
                                })
                            })
                    }
                    _ => Err("command not found".to_string()),
                };

                match result {
                    Ok(None) => (),
                    Ok(Some(content)) | Err(content) => {
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
                            println!("Cannot respond to slash command: {why}");
                        }
                    }
                };
            }
            Interaction::MessageComponent(interaction) => {
                match interaction.data.custom_id.as_str() {
                    "delete_letter" => commands::delete::handle_button(&interaction, &ctx).await,
                    custom_id => println!("Message component interaction not found: {custom_id}"),
                };
                // println!("a message component interaction arrived: {interaction:?}");
            }
            Interaction::ModalSubmit(mut interaction) => {
                match interaction.data.custom_id.as_str() {
                    "delete_modal" => {
                        commands::delete::handle_modal(
                            &mut interaction,
                            &ctx,
                            &mut self.db_pool.get().unwrap(),
                        )
                        .await
                    }
                    _ => (),
                }
            }
            Interaction::Autocomplete(interaction) => {
                commands::send::complete(&interaction, &ctx, &mut self.db_pool.get().unwrap())
                    .await
                    .unwrap();
            }
            _ => (),
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
                .create_application_command(|command| commands::add_recipient::register(command))
        })
        .await
        .expect("able to set application commands");

        println!(
            "I now have the following guild slash commands: {}",
            commands
                .iter()
                .map(|command| format!(
                    "\n- \"{}\" ({} options): {}",
                    command.name,
                    command.options.len(),
                    command.description
                ))
                .reduce(|acc, val| acc + &val)
                .unwrap()
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    {
        use diesel::prelude::*;
        use model::Recipient;
        use schema::recipients::dsl::recipients;

        let conn = &mut SqliteConnection::establish(&database_url).unwrap();

        run_migrations(conn).unwrap();

        if let Ok(var) = env::var("RECIPIENTS") {
            _ = diesel::delete(recipients).execute(conn).unwrap();
            diesel::insert_into(recipients)
                .values(
                    var.split(':')
                        .map(|name| Recipient {
                            fullname: name.replace('_', " "),
                            is_real: false,
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(conn)
                .unwrap();
        } else {
            println!("No default recipients specified, not resetting database.")
        }
    }

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler {
            db_pool: Pool::builder()
                .test_on_check_out(true)
                .build(ConnectionManager::<SqliteConnection>::new(database_url))
                .expect("Could not build connection pool"),
            letters_allowed: AtomicBool::new(true),
        })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
