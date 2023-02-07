use diesel::prelude::*;
use serenity::{
    model::prelude::{
        component::{ActionRowComponent, InputText, InputTextStyle},
        interaction::{
            message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
            InteractionResponseType,
        },
        MessageId,
    },
    prelude::Context,
};

use crate::{model::Letter, schema::letters::all_columns};

pub async fn handle_button(interaction: &MessageComponentInteraction, ctx: &Context) {
    let Some(member) = &interaction.member else {
        return;
    };

    if !member
        .permissions
        .expect("member should have permissions")
        .manage_messages()
    {
        interaction
            .create_interaction_response(ctx, |response| {
                response.interaction_response_data(|data| {
                    data.content(
                        "You aren't allowed to do this. (Manage Messages permission required)",
                    )
                    .ephemeral(true)
                })
            })
            .await
            .unwrap();
        return;
    };

    interaction
        .create_interaction_response(ctx, |response| {
            response
                .kind(InteractionResponseType::Modal)
                .interaction_response_data(|data| {
                    data.custom_id("delete_modal")
                        .title("You are about to delete a Valentines letter!")
                        .components(|components| {
                            components.create_action_row(|row| {
                                row.create_input_text(|input| {
                                    input
                                        .custom_id(interaction.message.id)
                                        .label("Just making sure!")
                                        .required(false)
                                        .placeholder("Don't make a mistake!")
                                        .style(InputTextStyle::Short)
                                })
                            })
                        })
                })
        })
        .await
        .unwrap();
}

pub async fn handle_modal(
    interaction: &mut ModalSubmitInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) {
    let ActionRowComponent::InputText(InputText { custom_id: message_id, ..}) = interaction.data.components.get(0).unwrap().components.get(0).unwrap() else {panic!()};
    let message_id = MessageId(message_id.parse().unwrap());

    let deleted = delete_letter(message_id, db_conn).expect("Can delete letter");
    interaction
        .message
        .as_mut()
        .unwrap()
        .edit(ctx, |edit| {
            use ellipse::Ellipse;
            edit
            .components(|components| components)
            .embed(|e| {
                e.title(if deleted.anon {
                    format!(
                        "Deleted: Sent anonymously by {} to {}",
                        deleted.sender, deleted.recipient
                    )
                } else {
                    format!("Deleted: Sent by {} to {}", deleted.sender, deleted.recipient)
                })
                .description(&deleted.content.as_str().truncate_ellipse(50))
                .field("Deleted", format!(
                    "by {} at {}",
                    interaction.user.name,
                    chrono::prelude::Utc::now().to_rfc3339()
                ), false)
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
                .color((255, 0, 0))
            })
        })
        .await
        .unwrap();

    interaction
        .create_interaction_response(ctx, |response| {
            response
                .interaction_response_data(|data| data.content("Deleted a message").ephemeral(true))
        })
        .await
        .unwrap();
}

fn delete_letter(to_delete: MessageId, conn: &mut SqliteConnection) -> Result<Letter, String> {
    use crate::schema::letters::dsl::{letters, message_id};

    diesel::delete(letters.filter(message_id.eq(to_delete.to_string())))
        .returning(all_columns)
        .get_result(conn)
        .map_err(|e| format!("Error {e}"))
}
