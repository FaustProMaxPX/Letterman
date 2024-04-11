// @generated automatically by Diesel CLI.

diesel::table! {
    t_post (id) {
        id -> Bigint,
        post_id -> Bigint,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        metadata -> Varchar,
        version -> Integer,
        prev_version -> Integer,
        create_time -> Timestamp,
        update_time -> Timestamp,
    }
}

diesel::table! {
    t_post_content (id) {
        id -> Bigint,
        post_id -> Bigint,
        version -> Integer,
        content -> Text,
        prev_version -> Integer,
        create_time -> Timestamp,
        update_time -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    t_post,
    t_post_content,
);
