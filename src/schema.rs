table! {
    inbox (rowid) {
        rowid -> Integer,
        id -> Text,
        json -> Text,
    }
}

table! {
    posts (id) {
        id -> Integer,
        uri_name -> Text,
        datetime -> Integer,
        title -> Text,
        body -> Binary,
    }
}

allow_tables_to_appear_in_same_query!(
    inbox,
    posts,
);
