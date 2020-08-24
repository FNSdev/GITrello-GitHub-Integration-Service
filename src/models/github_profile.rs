#[derive(Queryable)]
pub struct GithubProfile {
    pub id: i32,
    pub user_id: i64,
    pub github_user_id: i64,
    pub github_login: String,
    pub access_token: String,
}
