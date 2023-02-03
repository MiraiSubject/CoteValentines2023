// @generated automatically by Diesel CLI.

diesel::table! {
    letters (id) {
        id -> Nullable<Integer>,
        recipient -> Text,
        sender -> Text,
        anon -> Bool,
        content -> Text,
    }
}
