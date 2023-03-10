use std::env;

use diesel::prelude::*;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction,
                autocomplete::AutocompleteInteraction,
            },
            ChannelId, Message,
        },
        Permissions,
    },
    prelude::Context,
};

use crate::commands::log_letters::log_letter;

use super::{as_boolean, as_string};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("sendletter")
        .description("Send a letter to your valentine! (max 2 letters allowed)")
        .create_option(|option| {
            option
                .name("recipient")
                .description("The mod or heroine whom you want to send a valentine's letter to")
                .kind(CommandOptionType::String)
                .min_length(1)
                .max_length(20)
                .required(true)
                .set_autocomplete(true)
        })
        .create_option(|option| {
            option
                .name("letter")
                .description("The letter that you want to send to this person!")
                .kind(CommandOptionType::String)
                .min_length(100)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("anonymous")
                .description("Do you want to send this message anonymously?")
                .kind(CommandOptionType::Boolean)
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

        "Thank you for your message, it has been recorded.".to_owned()
    } else {
        "You have already sent two messages.".to_owned()
    }))
}

pub async fn forbidden(
) -> Result<Option<String>, String> {
    Err("Letter submissions are disabled".to_string())
}

pub async fn complete(
    interaction: &AutocompleteInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) -> Result<(), &'static str> {
    use crate::schema::recipients::dsl::{fullname, recipients};

    let up_to_now = as_string(
        interaction
            .data
            .options
            .get(0)
            .ok_or("No recipient found")?
            .resolved
            .as_ref()
            .ok_or("Expected recipient object")?,
    )
    .map_err(|_| "Recipient is not string")?;

    interaction
        .create_autocomplete_response(ctx, |response| {
            let names: Vec<String> = recipients
                .filter(fullname.like(format!("%{up_to_now}%")))
                .select(fullname)
                .limit(25)
                .load(db_conn)
                .unwrap();

            for name in names {
                response.add_string_choice(&name, &name);
            }

            response
        })
        .await
        .unwrap();
    Ok(())
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

        let recipient = as_string(
            options
                .get(0)
                .ok_or(ParseOptionsError("No recipient found"))?
                .resolved
                .as_ref()
                .ok_or(ParseOptionsError("Expected recipient object"))?,
        )
        .map_err(|_| ParseOptionsError("Recipient is not string"))?;

        let letter = as_string(
            options
                .get(1)
                .ok_or(ParseOptionsError("No message contents count"))?
                .resolved
                .as_ref()
                .ok_or(ParseOptionsError("Letter object expected"))?,
        )
        .map_err(|_| ParseOptionsError("Letter is not string"))?;

        let is_anon = as_boolean(
            options
                .get(2)
                .ok_or(ParseOptionsError(
                    "We don't know if the user wants to send anonymously",
                ))?
                .resolved
                .as_ref()
                .ok_or(ParseOptionsError("Expected boolean object"))?,
        )
        .map_err(|_| ParseOptionsError("Anonymous is not boolean"))?;

        Ok(ValentineLetter {
            sender: user.name.clone(),
            recipient: recipient.to_string(),
            letter: letter.to_string(),
            anon: *is_anon,
            sender_id: value.user.id.to_string(),
        })
    }
}

struct DatabaseProblem;
