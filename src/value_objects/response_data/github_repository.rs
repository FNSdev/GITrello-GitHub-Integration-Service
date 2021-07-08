use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct GetGithubRepositoryResponse {
    pub name: String,
    pub owner: String,
}
