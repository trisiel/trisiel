table! {
    gitea_tokens (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        access_token -> Varchar,
        refresh_token -> Varchar,
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
    }
}

joinable!(gitea_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    gitea_tokens,
    users,
);
