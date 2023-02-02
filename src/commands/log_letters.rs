use serenity::{model::prelude::ChannelId, prelude::Context};

use super::send::ValentineLetter;

pub async fn log_letter(ctx: &Context, letter: &ValentineLetter) {
    let public_facing_channel_id = ChannelId(336524751860138005);
    let audit_channel_id = ChannelId(610201663382487065);

    let public_message = ChannelId::send_message(public_facing_channel_id, &ctx.http, |m| {
        m.embed(|e| {
            e.title(if letter.anon {
                format!("To {}", letter.recipient)
            } else {
                format!("From {} to {}", letter.sender, letter.recipient)
            })

            .description(&letter.letter)
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
        })
    })
    .await;

    if let Err(why) = public_message {
        println!("Error sending message: {:?}", why);
    }

    let audit_message = ChannelId::send_message(audit_channel_id, &ctx.http, |m| {
        m.embed(|e| {
            if letter.anon {
                e.title(format!("ANONYMOUSLY SENT: From {} to {}", letter.sender, letter.recipient));
            } else {
                e.title(format!("From {} to {}", letter.sender, letter.recipient));
            }

            e.description(&letter.letter)
                .footer(|f| f.text("2023 Classroom of the Elite Valentine's Event"))
        })
    })
    .await;

    if let Err(why) = audit_message {
        println!("Error sending message: {:?}", why);
    }
}
