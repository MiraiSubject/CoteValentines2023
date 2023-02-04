use diesel::prelude::*;
use diesel::SqliteConnection;
use serenity::builder::CreateEmbed;
use serenity::model::prelude::ChannelType;
use serenity::prelude::Context;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::application_command::{
                ApplicationCommandInteraction, CommandDataOptionValue,
            },
        },
        Permissions,
    },
};
use tokio::time::{sleep, Duration};

use crate::model::*;
use crate::schema::letters::dsl::*;

impl Letter {
    fn build_embed<'a>(&self, e: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        e.title(if self.anon {
            format!("To {}", self.recipient.clone())
        } else {
            format!("From {} to {}", self.sender.clone(), self.recipient.clone())
        })
        .description(self.content.clone())
        .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
    }
}

pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) -> Result<String, String> {
    if let CommandDataOptionValue::Channel(channel) = command
        .data
        .options
        .get(0)
        .ok_or("No channel found".to_owned())?
        .resolved
        .as_ref()
        .ok_or("Expected channel object".to_owned())?
    {
        let ChannelType::Text = channel.kind else {
            return Err("Bad channel type".to_owned())
        };

        let ten_seconds = Duration::from_millis(4000);

        for letter in letters
            .load::<Letter>(db_conn)
            .map_err(|_| "database error".to_owned())?
        {
            let channel_id = channel.id;
            channel_id
                .broadcast_typing(&ctx.http)
                .await
                .map_err(|e| format!("Error sending typing event.\n ```{e:?}```"))?;

            sleep(ten_seconds).await;

            let ret = channel_id
                .send_message(ctx, |m| m.embed(|embed| letter.build_embed(embed)))
                .await
                .map_err(|e| format!("Error sending a message:\n```{e:?}```"))?;

            dbg!(ret);
        };

        Ok("done".to_owned())
    } else {
        Err("No channel found".to_owned())
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("publish")
        .description("sends all letters to a channel")
        .create_option(|option| {
            option
                .name("target")
                .description("Channel to send to, defaults to current")
                .kind(CommandOptionType::Channel)
                .required(true)
        })
        .dm_permission(false)
        .default_member_permissions(Permissions::READ_MESSAGE_HISTORY)
}
