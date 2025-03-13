// @generated automatically by Diesel CLI.

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
