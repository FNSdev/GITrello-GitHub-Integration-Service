use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct NewBoardRepositoryRequest {
    pub board_id: i64,
    pub repository_id: i64,
}
