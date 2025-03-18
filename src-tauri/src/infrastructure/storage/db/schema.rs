// @generated automatically by Diesel CLI.

diesel::table! {
    clipboard_records (id) {
        id -> Text,
        device_id -> Text,
        local_file_path -> Nullable<Text>,
        remote_record_id -> Nullable<Text>,
        content_type -> Text,
        is_favorited -> Bool,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    devices (id) {
        id -> Text,
        ip -> Nullable<Text>,
        port -> Nullable<Integer>,
        server_port -> Nullable<Integer>,
        status -> Integer,
        self_device -> Bool,
        updated_at -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    clipboard_records,
    devices,
);
