use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NewBoardRepositoryRequest {
    pub board_id: i64,
    pub repository_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetBoardRepositoryQueryParams {
    pub board_id: i64,
}
