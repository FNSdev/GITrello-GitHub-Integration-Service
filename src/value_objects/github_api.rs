use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct APIError {
    pub message: String,
}


#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
}
