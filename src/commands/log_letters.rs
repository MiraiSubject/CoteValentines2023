use serenity::{
    model::prelude::{component::ButtonStyle, ChannelId, Message},
    prelude::Context,
};

use super::send::ValentineLetter;

pub async fn log_letter(
    ctx: &Context,
    letter: &ValentineLetter,
    audit_channel: ChannelId,
) -> serenity::Result<Message> {
    ChannelId::send_message(audit_channel, &ctx.http, |m| {
        m.embed(|embed| {
            embed
                .title(if letter.anon {
                    format!(
                        "Sent anonymously by {} to {}",
                        letter.sender, letter.recipient
                    )
                } else {
                    format!("Sent by {}", letter.sender)
                })
                .description(&letter.letter)
                .field("Author ID", &letter.sender_id, true)
                .footer(|f| f.text("2023 COTE April Fools Event"))
        })
        .components(|components| {
            components.create_action_row(|row| {
                row.create_button(|button| {
                    button
                        .custom_id("delete_letter")
                        .emoji('🗑')
                        .style(ButtonStyle::Danger)
                        .label("Delete")
                })
            })
        })
    })
    .await
}
