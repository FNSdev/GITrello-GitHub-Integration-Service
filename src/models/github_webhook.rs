use serde::{Deserialize, Serialize};

use crate::schema::github_webhook;

#[table_name = "github_webhook"]
#[derive(Debug, Clone, Identifiable, Serialize, Queryable)]
pub struct GithubWebhook {
    pub id: i32,
    pub webhook_id: i64,
    pub url: String,
    pub board_repository_id: i32,
}

#[table_name = "github_webhook"]
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
pub struct NewGithubWebhook {
    pub webhook_id: i64,
    pub url: String,
    pub board_repository_id: i32,
}
