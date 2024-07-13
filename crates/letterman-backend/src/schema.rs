// @generated automatically by Diesel CLI.

diesel::table! {
    t_github_post_record (id) {
        id -> Integer,
        post_id -> Bigint,
        version -> Integer,
        #[max_length = 255]
        path -> Varchar,
        #[max_length = 255]
        sha -> Varchar,
        #[max_length = 255]
        repository -> Varchar,
        #[max_length = 255]
        url -> Varchar,
        create_time -> Timestamp,
        update_time -> Timestamp,
    }
}

diesel::table! {
    t_post (id) {
        id -> Bigint,
        post_id -> Bigint,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        metadata -> Varchar,
        #[max_length = 256]
        version -> Varchar,
        #[max_length = 256]
        prev_version -> Varchar,
        create_time -> Timestamp,
        update_time -> Timestamp,
        head -> Bool,
    }
}

diesel::table! {
    t_post_content (id) {
        id -> Bigint,
        post_id -> Bigint,
        #[max_length = 256]
        version -> Varchar,
        content -> Text,
        #[max_length = 256]
        prev_version -> Varchar,
        create_time -> Timestamp,
        update_time -> Timestamp,
        head -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    t_github_post_record,
    t_post,
    t_post_content,
);
