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
