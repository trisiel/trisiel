table! {
    gitea_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        access_token -> Varchar,
        refresh_token -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    handlers (id) {
        id -> Uuid,
        user_id -> Uuid,
        human_name -> Varchar,
        current_version -> Varchar,
        async_impl -> Nullable<Bool>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        salutation -> Varchar,
        is_admin -> Bool,
        is_locked -> Bool,
        tier -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    gitea_tokens,
    handlers,
    tokens,
    users,
);
