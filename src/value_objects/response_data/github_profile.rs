use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GithubProfileResponse {
    pub id: i32,
    pub user_id: String,
    pub github_user_id: String,
    pub github_login: String,
}
