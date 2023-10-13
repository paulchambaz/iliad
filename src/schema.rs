// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (username) {
        username -> Text,
        password -> Text,
        key -> Text,
    }
}

diesel::table! {
    audiobooks (hash) {
        hash -> Text,
        title -> Text,
        author -> Text,
        date -> Text,
        description -> Text,
        genres -> Text,
        duration -> Integer,
        size -> Integer,
        path -> Text,
    }
}

diesel::table! {
    positions (hash, username) {
        hash -> Text,
        username -> Text,
        file -> Text,
        position -> Integer,
        last_modified -> Date,
    }
}

diesel::allow_tables_to_appear_in_same_query!(accounts, audiobooks, positions,);
