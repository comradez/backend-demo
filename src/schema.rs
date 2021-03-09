table! {
    message (id) {
        id -> Integer,
        user -> Integer,
        title -> Text,
        content -> Text,
        pub_date -> Timestamp,
    }
}

table! {
    user (id) {
        id -> Integer,
        name -> Text,
        register_date -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    message,
    user,
);
