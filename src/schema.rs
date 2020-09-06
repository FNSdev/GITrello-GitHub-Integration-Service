table! {
    board_repository (id) {
        id -> Int4,
        board_id -> Int8,
        repository_name -> Varchar,
        repository_owner -> Varchar,
    }
}

table! {
    github_profile (id) {
        id -> Int4,
        user_id -> Int8,
        github_user_id -> Int8,
        github_login -> Varchar,
        access_token -> Varchar,
    }
}

table! {
    github_webhook (id) {
        id -> Int4,
        webhook_id -> Int8,
        url -> Varchar,
        board_repository_id -> Int4,
    }
}

joinable!(github_webhook -> board_repository (board_repository_id));

allow_tables_to_appear_in_same_query!(
    board_repository,
    github_profile,
    github_webhook,
);
