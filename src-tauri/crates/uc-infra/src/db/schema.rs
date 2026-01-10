// @generated automatically by Diesel CLI.

diesel::table! {
    blob (blob_id) {
        blob_id -> Nullable<Text>,
        storage_path -> Text,
        size_bytes -> Integer,
        content_hash -> Text,
        encryption_algo -> Nullable<Text>,
        created_at_ms -> Integer,
    }
}

diesel::table! {
    clipboard_entry (entry_id) {
        entry_id -> Nullable<Text>,
        event_id -> Text,
        created_at_ms -> Integer,
        title -> Nullable<Text>,
        total_size -> Integer,
        pinned -> Bool,
        deleted_at_ms -> Nullable<Integer>,
    }
}

diesel::table! {
    clipboard_event (event_id) {
        event_id -> Nullable<Text>,
        captured_at_ms -> Integer,
        source_device -> Text,
        snapshot_hash -> Text,
    }
}

diesel::table! {
    clipboard_selection (entry_id) {
        entry_id -> Nullable<Text>,
        primary_rep_id -> Text,
        preview_rep_id -> Text,
        paste_rep_id -> Text,
        policy_version -> Text,
    }
}

diesel::table! {
    clipboard_snapshot_representation (id) {
        id -> Nullable<Text>,
        event_id -> Text,
        format_id -> Text,
        mime_type -> Nullable<Text>,
        size_bytes -> Integer,
        inline_data -> Nullable<Binary>,
        blob_id -> Nullable<Text>,
    }
}

diesel::table! {
    t_device (id) {
        id -> Text,
        name -> Text,
        created_at -> BigInt,
    }
}

diesel::joinable!(clipboard_entry -> clipboard_event (event_id));
diesel::joinable!(clipboard_selection -> clipboard_entry (entry_id));
diesel::joinable!(clipboard_snapshot_representation -> blob (blob_id));
diesel::joinable!(clipboard_snapshot_representation -> clipboard_event (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    blob,
    clipboard_entry,
    clipboard_event,
    clipboard_selection,
    clipboard_snapshot_representation,
    t_device,
);
