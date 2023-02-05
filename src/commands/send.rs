use diesel::prelude::*;

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
    prelude::Context,
};

use crate::commands::log_letters::log_letter;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("sendletter")
        .description("A ping command")
        .create_option(|option| {
            option
                .name("recipient")
                .description("The mod or heroine whom you want to send a valentine's letter to")
                .kind(CommandOptionType::String)
                .min_length(1)
                .max_length(20)
                .required(true)
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
    use crate::schema::letters::dsl::*;

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
) -> Result<(), DatabaseProblem> {
    use crate::model::*;
    use crate::schema::letters::dsl::*;

    let letter = NewLetter {
        sender: &letter.sender,
        recipient: &letter.recipient,
        anon: letter.anon,
        content: &letter.letter,
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
    let user = &command.user;
    let options = &command.data.options;

    let recipient = options
        .get(0)
        .expect("No recipient found")
        .resolved
        .as_ref()
        .expect("Expected recipient object");

    let letter = options
        .get(1)
        .expect("No message contents count")
        .resolved
        .as_ref()
        .expect("Letter object expected");

    let anon_option = options
        .get(2)
        .expect("We don't know if the user wants to send anonymously")
        .resolved
        .as_ref()
        .expect("Expected boolean object");

    fn as_string(optionval: &CommandDataOptionValue) -> Result<&String, ()> {
        if let CommandDataOptionValue::String(stringval) = optionval {
            Ok(stringval)
        } else {
            Err(())
        }
    }

    fn as_boolean(optionval: &CommandDataOptionValue) -> Result<&bool, ()> {
        if let CommandDataOptionValue::Boolean(val) = optionval {
            Ok(val)
        } else {
            Err(())
        }
    }

    let recipient = as_string(recipient).unwrap();
    let letter = as_string(letter).unwrap();
    let is_anon = as_boolean(anon_option).unwrap();

    let letter = ValentineLetter {
        sender: user.name.clone(),
        recipient: recipient.to_string(),
        letter: letter.to_string(),
        anon: *is_anon,
    };

    let can_send = user_can_send_letter(db_conn, &letter)
        .map_err(|_| "Something went very wrong.".to_owned())?;

    Ok(Some(if can_send {
        add_letter_to_user(db_conn, &letter)
            .map_err(|_| "Something went very wrong.".to_owned())?;

        log_letter(ctx, &letter).await;

        "Thank you for your message, it has been recorded.".to_owned()
    } else {
        "You have already sent two messages.".to_owned()
    }))
}

pub struct ValentineLetter {
    pub sender: String,
    pub recipient: String,
    pub letter: String,
    pub anon: bool,
}

struct DatabaseProblem;
