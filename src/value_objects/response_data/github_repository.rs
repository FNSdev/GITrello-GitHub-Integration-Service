use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct GetGithubRepositoryResponse {
    pub id: String,
    pub name: String,
}
