use serenity::{
    model::prelude::{component::ButtonStyle, ChannelId, Message},
    prelude::Context,
};

use super::send::ValentineLetter;

pub async fn log_letter(ctx: &Context, letter: &ValentineLetter) -> serenity::Result<Message> {
    let audit_channel_id = ChannelId(610201663382487065);

    ChannelId::send_message(audit_channel_id, &ctx.http, |m| {
        m.embed(|embed| {
            embed
                .title(if letter.anon {
                    format!(
                        "ANONYMOUSLY SENT: From {} to {}",
                        letter.sender, letter.recipient
                    )
                } else {
                    format!("From {} to {}", letter.sender, letter.recipient)
                })
                .description(&letter.letter)
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
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
