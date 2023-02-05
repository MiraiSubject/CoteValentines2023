// @generated automatically by Diesel CLI.

diesel::table! {
    letters (id) {
        id -> Integer,
        recipient -> Text,
        sender -> Text,
        anon -> Bool,
        content -> Text,
        message_id -> Text,
        sender_id -> Text,
    }
}
