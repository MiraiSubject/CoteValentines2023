use diesel::prelude::*;
use serenity::{
    model::prelude::{interaction::message_component::MessageComponentInteraction, MessageId},
    prelude::Context,
};

pub async fn run(
    mut interaction: MessageComponentInteraction,
    ctx: &Context,
    db_conn: &mut SqliteConnection,
) {
    delete_letter(interaction.message.id, db_conn).unwrap();
    interaction.message.edit(ctx, |edit| {
        edit.content(
            format!("deleted by {} at {}", interaction.user.name, chrono::prelude::Utc::now().to_rfc3339())
        ).components(|components| components)
    }).await.unwrap();
}

fn delete_letter(to_delete: MessageId, conn: &mut SqliteConnection) -> Result<bool, String> {
    use crate::schema::letters::dsl::*;

    diesel::delete(letters.filter(message_id.eq(to_delete.to_string())))
        .execute(conn)
        .map_err(|e| {
            format!("Error {e}")
        })
        .map(|deleted_count| deleted_count != 0)
        
}
