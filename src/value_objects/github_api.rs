use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct APIError {
    pub message: String,
}


#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Repository {
    pub id: i64,
    pub full_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Webhook {
    pub id: i64,
}

#[derive(Serialize, Debug)]
pub struct CreateWebhook {
    pub config: CreateWebhookConfig,
    pub events: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct CreateWebhookConfig {
    pub url: String,
    pub content_type: String,
}
