use crate::db::model::{Letter as DbLetter, User as DbUser};
use crate::db::Database;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::user::User;

pub struct ValentineLetter {
    pub sender: String,
    pub recipient: String,
    pub letter: String,
    pub anon: bool,
}

impl ValentineLetter {
    pub fn to_db_letter(&self) -> DbLetter {
        DbLetter {
            anon: self.anon.clone(),
            recipient: self.recipient.clone(),
            content: self.letter.clone(),
        }
    }
}

pub fn run<DB>(
    options: &[CommandDataOption],
    user: &User,
    user_db: DB,
) -> Result<(ValentineLetter, String), String>
where
    DB: Database<DbUser>,
{
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

    // println!("The letter they sent is: {}", letter);
    // println!("{} sent Valentine's Letter to {}", user.name, recipient);
    // if *is_anon {
    //     println!("This user sent the message anonymously");
    // } else {
    //     println!("This dude did not send the message anonymously");
    // }

    let letter = ValentineLetter {
        sender: user.name.clone(),
        recipient: recipient.to_string(),
        letter: letter.to_string(),
        anon: *is_anon,
    };

    user_db
        .get(&user.id.to_string())
        .map_or_else(
            // if the user doesnt exist, create a new user
            |_| {
                dbg!("user doesnt exist yet");
                Ok(DbUser {
                    user_id: user.id.to_string(),
                    letters: (Some(letter.to_db_letter()), None),
                })
            },
            // if the user exists, try to add the new letter to the user
            |user| user.letter_added(letter.to_db_letter()),
        )
        .map(|user| {
            // if the letter can be (and was) added to the user
            dbg!(&user);
            // save the user object:
            user_db.save(&user).map_err(|_| "boohoo").unwrap();
            (
                letter,
                "Thank you for your message, it as been recorded".to_owned(),
            )
        })
        .map_err(|_| {
            dbg!("user already has 2 letters");
            // if the user already has two letters
            "You have already sent two messages.".to_owned()
        })
}

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
}
