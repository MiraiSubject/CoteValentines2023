use diesel::insert_into;
use diesel::prelude::*;
use diesel::SqliteConnection;

use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::application_command::ApplicationCommandInteraction,
        },
        Permissions,
    },
    prelude::Context,
};

use super::{as_boolean, as_string};

use crate::model::Recipient;
use crate::schema::recipients::dsl::recipients;

pub async fn run(
    command: &ApplicationCommandInteraction,
    _ctx: &Context,
    db_conn: &mut SqliteConnection,
) -> Result<Option<String>, String> {
    let new: Recipient = command.try_into()?;
    insert_into(recipients)
        .values(&new)
        .execute(db_conn)
        .map_err(|e| format!("Something went wrong while adding person: \n{e}"))?;

    Ok(Some(format!(
        "Done adding {} person {}",
        {
            if new.is_real {
                "real"
            } else {
                "fictional"
            }
        },
        new.fullname
    )))
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("add_recipient")
        .description("adds a recipient to the autocomplete list")
        .create_option(|option| {
            option
                .name("name")
                .description("name of the person to add")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("is_real")
                .description("whether this person is a real human")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
        .dm_permission(false)
        .default_member_permissions(Permissions::ADMINISTRATOR)
}

impl TryFrom<&ApplicationCommandInteraction> for Recipient {
    type Error = String;

    fn try_from(value: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        let options = &value.data.options;
        Ok(Self {
            fullname: as_string(
                options
                    .get(0)
                    .ok_or("No name".to_owned())?
                    .resolved
                    .as_ref()
                    .ok_or("Name object expected".to_owned())?,
            )
            .map_err(|_| "Name is not string".to_owned())?.clone(),
            is_real: as_boolean(
                options
                    .get(1)
                    .ok_or("we dont know if the person is real!".to_string())?
                    .resolved
                    .as_ref()
                    .ok_or("Expected boolean object".to_owned())?,
            )
            .map_err(|_| ("Reality is not boolean").to_owned())?
            .to_owned(),
        })
    }
}
