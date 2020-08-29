use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BoardRepositoryResponse {
    pub id: i32,
    pub board_id: String,
    pub repository_name: String,
    pub repository_owner: String,
}
