use serde::{Deserialize, Serialize};

use crate::schema::github_profile;

#[derive(Debug, Clone, Serialize, Queryable)]
pub struct GithubProfile {
    pub id: i32,
    pub user_id: i64,
    pub github_user_id: i64,
    pub github_login: String,
    pub access_token: String,
}

#[table_name = "github_profile"]
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
pub struct NewGithubProfile {
    pub user_id: i64,
    pub github_user_id: i64,
    pub github_login: String,
    pub access_token: String,
}
