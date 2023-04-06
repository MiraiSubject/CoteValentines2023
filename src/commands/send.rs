use std::env;

use diesel::prelude::*;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction,
            },
            ChannelId, Message,
        },
        Permissions,
    },
    prelude::Context,
};

use crate::commands::log_letters::log_letter;

use super::{as_string};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("sendsubmission")
        .description("Submit your answer here (2 answers max)")
        .create_option(|option| {
            option
                .name("submission")
                .description("The submission you want to turn in")
                .kind(CommandOptionType::String)
                .min_length(5)
                .required(true)
        })
        .dm_permission(true)
        .default_member_permissions(Permissions::SEND_MESSAGES)
}

fn user_can_send_letter(
    conn: &mut SqliteConnection,
    letter: &ValentineLetter,
) -> Result<bool, DatabaseProblem> {
    use crate::schema::letters::dsl::{letters, sender};

    let letter_count: i64 = letters
        .filter(sender.eq(&letter.sender))
        .count()
        .get_result(conn)
        .map_err(|_| DatabaseProblem)?;

    if letter_count >= 2 {
        Ok(false)
    } else {
        Ok(true)
    }
}

fn add_letter_to_user(
    conn: &mut SqliteConnection,
    letter: &ValentineLetter,
    log_message: Option<&Message>,
) -> Result<(), DatabaseProblem> {
    use crate::model::NewLetter;
    use crate::schema::letters::dsl::letters;

    let letter = NewLetter {
        sender: &letter.sender,
        recipient: &letter.recipient,
        anon: letter.anon,
        content: &letter.letter,
        message_id: log_message.map(|msg| msg.id.to_string()),
        sender_id: &letter.sender_id,
    };

    diesel::insert_into(letters)
        .values(&letter)
        .execute(conn)
        .map_err(|_| DatabaseProblem)?;

    Ok(())
}

pub async fn run(
    command: &ApplicationCommandInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) -> Result<Option<String>, String> {
    let letter: ValentineLetter = command
        .try_into()
        .map_err(|e| format!("Error while parsing arguments: {e:?}"))?;

    let can_send = user_can_send_letter(db_conn, &letter)
        .map_err(|_| "Something went very wrong.".to_owned())?;

    Ok(Some(if can_send {
        let log_message = if let Some(log_channel) = env::var("AUDIT_CHANNEL_ID")
            .map_err(|e| e.to_string())
            .and_then(|id_as_str| id_as_str.parse::<u64>().map_err(|e| e.to_string()))
            .map(ChannelId)
            .map_or_else(
                |e| {
                    println!("letter not logged: no audit channel specified!\n{e}");
                    None
                },
                Some,
            )
        {
            Some(
                log_letter(ctx, &letter, log_channel)
                    .await
                    .map_err(|_| "Something went wrong")?
                )
        } else {
            None
        };

        add_letter_to_user(db_conn, &letter, log_message.as_ref())
            .map_err(|_| "Something went very wrong.".to_owned())?;

        "Thank you for your submission.".to_owned()
    } else {
        "You have already sent two messages.".to_owned()
    }))
}

pub async fn forbidden(
) -> Result<Option<String>, String> {
    Err("Letter submissions are disabled".to_string())
}

pub struct ValentineLetter {
    pub sender: String,
    pub sender_id: String,
    pub recipient: String,
    pub letter: String,
    pub anon: bool,
}

#[derive(Debug)]
pub struct ParseOptionsError(&'static str);

impl TryFrom<&ApplicationCommandInteraction> for ValentineLetter {
    type Error = ParseOptionsError;

    fn try_from(value: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        let user = &value.user;
        let options = &value.data.options;

        let letter = as_string(
            options
                .get(0)
                .ok_or(ParseOptionsError("No message contents count"))?
                .resolved
                .as_ref()
                .ok_or(ParseOptionsError("Letter object expected"))?,
        )
        .map_err(|_| ParseOptionsError("Letter is not string"))?;

        Ok(ValentineLetter {
            sender: user.name.clone(),
            recipient: "".to_owned(),
            letter: letter.to_string(),
            anon: false,
            sender_id: value.user.id.to_string(),
        })
    }
}

struct DatabaseProblem;
