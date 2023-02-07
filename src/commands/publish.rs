use diesel::prelude::*;
use diesel::SqliteConnection;
use serenity::builder::CreateEmbed;

use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::interaction::application_command::ApplicationCommandInteraction, Permissions,
    },
    prelude::Context,
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
        .colour({
            use random_color::{Color, RandomColor};

            let colour = RandomColor::new().hue(Color::Pink).to_rgb_array();
            (colour[0], colour[1], colour[2])
        })
    }
}

pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) -> Result<Option<String>, String> {
    // first, deferred reply to be allowed to take longer:
    command
        .create_interaction_response(ctx, |response| {
            response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
        .map_err(|_| "cant send a response???")?;

    let found_letters = letters
        .load::<Letter>(db_conn)
        .map_err(|_| "database error".to_owned())?;

    const MAX_RUNTIME: Duration = Duration::from_secs(60 * 10);
    const MAX_DELAY_PER_LETTER: Duration = Duration::from_secs(5);

    let max_delay = MAX_DELAY_PER_LETTER.min(Duration::from_millis(
        (MAX_RUNTIME.as_millis() / found_letters.len() as u128) as u64,
    ));

    let channel_id = command.channel_id;

    let typing = channel_id
        .start_typing(&ctx.http)
        .map_err(|e| format!("Error sending typing event.\n ```{e:?}```"))?;

    for letter in found_letters {
        // wait a bit
        sleep(max_delay).await;
        // send embed and stop typing

        let ret = channel_id
            .send_message(ctx, |m| m.embed(|embed| letter.build_embed(embed)))
            .await
            .map_err(|e| format!("Error sending a message:\n```{e:?}```"))?;

        dbg!(ret);
    }

    let _ = typing.stop();

    command
        .edit_original_interaction_response(ctx, |edit| edit.content("Done"))
        .await
        .unwrap();

    Ok(None)
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("publish")
        .description("sends all letters to a channel")
        .dm_permission(false)
        .default_member_permissions(Permissions::MANAGE_MESSAGES)
}
