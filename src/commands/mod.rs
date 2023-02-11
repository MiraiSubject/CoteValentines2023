pub mod add_recipient;
pub mod delete;
pub mod log_letters;
pub mod publish;
pub mod send;
pub mod allow_letters;

use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
pub fn as_string(optionval: &CommandDataOptionValue) -> Result<&String, ()> {
    if let CommandDataOptionValue::String(stringval) = optionval {
        Ok(stringval)
    } else {
        Err(())
    }
}

pub fn as_boolean(optionval: &CommandDataOptionValue) -> Result<&bool, ()> {
    if let CommandDataOptionValue::Boolean(val) = optionval {
        Ok(val)
    } else {
        Err(())
    }
}
