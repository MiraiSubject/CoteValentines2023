use diesel::prelude::*;
use serenity::{
    model::prelude::{interaction::message_component::MessageComponentInteraction, MessageId},
    prelude::Context,
};

use crate::{model::Letter, schema::letters::all_columns};

pub async fn run(
    mut interaction: MessageComponentInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) {
    let deleted = delete_letter(interaction.message.id, db_conn).expect("Can delete letter");
    interaction
        .message
        .edit(ctx, |edit| {
            use ellipse::Ellipse;
            edit.content(format!(
                "deleted by {} at {}",
                interaction.user.name,
                chrono::prelude::Utc::now().to_rfc3339()
            ))
            .components(|components| components)
            .embed(|e| {
                e.title(if deleted.anon {
                    format!(
                        "DELETED: ANONYMOUSLY SENT: From {} to {}",
                        deleted.sender, deleted.recipient
                    )
                } else {
                    format!("DELETED: From {} to {}", deleted.sender, deleted.recipient)
                })
                .description(&deleted.content.as_str().truncate_ellipse(50))
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
                .color((255, 0, 0))
            })
        })
        .await
        .unwrap();
}

fn delete_letter(to_delete: MessageId, conn: &mut SqliteConnection) -> Result<Letter, String> {
    use crate::schema::letters::dsl::*;

    diesel::delete(letters.filter(message_id.eq(to_delete.to_string())))
        .returning(all_columns)
        .get_result(conn)
        .map_err(|e| format!("Error {e}"))
}
