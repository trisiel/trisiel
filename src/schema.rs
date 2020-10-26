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
