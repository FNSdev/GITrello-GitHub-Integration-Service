table! {
    board_repository (id) {
        id -> Int4,
        board_id -> Int8,
        repository_id -> Int8,
        github_profile_id -> Int4,
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

joinable!(board_repository -> github_profile (github_profile_id));

allow_tables_to_appear_in_same_query!(
    board_repository,
    github_profile,
);
