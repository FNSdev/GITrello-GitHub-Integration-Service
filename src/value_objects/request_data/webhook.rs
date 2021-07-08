use serde::{Deserialize};

use crate::value_objects::github_api::{Issue, Repository};

#[derive(Deserialize)]
pub struct IssueWebhookRequest {
    pub action: String,
    pub issue: Issue,
    pub repository: Repository,
}
