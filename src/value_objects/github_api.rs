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

impl Repository {
    pub fn split_full_name(&self) -> (&str, &str) {
        let parts: Vec<&str> = self.full_name.splitn(2, "/").collect();
        (parts[0], parts[1])
    }
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


#[derive(Deserialize, Debug)]
pub struct Issue {
    pub html_url: String,
    pub title: String,
    pub body: String,
}
