use serenity::{
    model::prelude::{ChannelId, Message},
    prelude::Context,
};

use super::send::ValentineLetter;

pub async fn log_letter(ctx: &Context, letter: &ValentineLetter) -> serenity::Result<Message> {
    let audit_channel_id = ChannelId(610201663382487065);

    ChannelId::send_message(audit_channel_id, &ctx.http, |m| {
        m.embed(|e| {
            if letter.anon {
                e.title(format!(
                    "ANONYMOUSLY SENT: From {} to {}",
                    letter.sender, letter.recipient
                ));
            } else {
                e.title(format!("From {} to {}", letter.sender, letter.recipient));
            }

            e.description(&letter.letter)
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
        })
    })
    .await
}
