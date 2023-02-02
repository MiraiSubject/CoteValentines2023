mod commands;
mod db;

use commands::log_letters::log_letter;
use db::{model::User, Database};

use dotenv::dotenv;
use jammdb::DB;
use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler {
    db_conn: DB,
}

#[derive(Debug)]
enum DbError {
    WriteError,
    ReadError,
}

impl Database<User> for &Handler {
    type DbError = DbError;

    fn get(&self, user_id: &str) -> Result<User, Self::DbError> {
        let tx = self.db_conn.tx(false).unwrap();
        let ret = User::get(
            &tx.get_bucket("users").map_err(|_| DbError::ReadError)?,
            user_id,
        )
        .map_err(|_| DbError::ReadError);

        tx.commit().map_err(|_| DbError::ReadError)?;

        ret
    }

    fn save(&self, user: &User) -> Result<(), Self::DbError> {
        let tx = self.db_conn.tx(true).unwrap();
        let ret = user.save(
            &tx.get_or_create_bucket("users").unwrap(),
        )
        .map_err(|_| DbError::WriteError);
        tx.commit().unwrap();
        ret
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                // "ping" => commands::ping::run(&command.data.options),
                "sendletter" => {
                    match commands::send::run(&command.data.options, &command.user, self) {
                        Ok((letter, message)) => {
                            log_letter(&ctx, &letter).await;
                            message
                        }
                        Err(message) => message,
                    }
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                // .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::send::register(command))
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

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler {
            db_conn: DB::open("runtime-database.db").unwrap(),
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
