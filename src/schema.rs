// @generated automatically by Diesel CLI.

diesel::table! {
    letters (id) {
        id -> Integer,
        recipient -> Text,
        sender -> Text,
        anon -> Bool,
        content -> Text,
        message_id -> Nullable<Text>,
        sender_id -> Text,
    }
}

diesel::table! {
    recipients (fullname) {
        fullname -> Text,
        is_real -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    letters,
    recipients,
);
