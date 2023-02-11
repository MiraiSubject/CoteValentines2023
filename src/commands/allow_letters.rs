use diesel::prelude::*;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::interaction::application_command::ApplicationCommandInteraction, Permissions,
    },
    prelude::Context,
};

use super::as_boolean;

pub async fn run(
    interaction: &ApplicationCommandInteraction,
    _ctx: &Context,
    _db_conn: &mut SqliteConnection,
) -> Result<Option<(bool, String)>, String> {
    as_boolean(
        interaction
            .data
            .options
            .get(0)
            .ok_or("No option found")?
            .resolved
            .as_ref()
            .ok_or("Expected boolean")?,
    )
    .map(|val| {
        Some((
            *val,
            format!(
                "Set letters to {}",
                if *val { "allowed" } else { "not allowed" }
            ),
        ))
    })
    .map_err(|_| "Something went wrong".to_string())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("allow_letters")
        .description("sets whether letters are allowed")
        .create_option(|option| {
            option
                .kind(serenity::model::prelude::command::CommandOptionType::Boolean)
                .name("allowed")
                .description("whether to allow letters")
                .required(true)
        })
        .dm_permission(false)
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
