// @generated automatically by Diesel CLI.

diesel::table! {
    t_clipboard_item (id) {
        id -> Text,
        record_id -> Text,
        index_in_record -> Integer,
        content_type -> Text,
        content_hash -> Text,
        blob_id -> Nullable<Text>,
        size -> Nullable<Integer>,
        mime -> Nullable<Text>,
    }
}

diesel::table! {
    t_clipboard_record (id) {
        id -> Text,
        source_device_id -> Text,
        origin -> Text,
        record_hash -> Text,
        item_count -> Integer,
        created_at -> BigInt,
        deleted_at -> Nullable<BigInt>,
    }
}

diesel::table! {
    t_device (id) {
        id -> Text,
        name -> Text,
        created_at -> BigInt,
    }
}

diesel::joinable!(t_clipboard_item -> t_clipboard_record (record_id));

diesel::allow_tables_to_appear_in_same_query!(
    t_clipboard_item,
    t_clipboard_record,
    t_device,
);
