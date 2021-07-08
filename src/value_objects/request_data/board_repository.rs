use serde::{Deserialize};

use crate::value_objects::request_data::utils::i64_from_str;

#[derive(Debug, Deserialize)]
pub struct NewBoardRepositoryRequest {
    #[serde(deserialize_with = "i64_from_str")]
    pub board_id: i64,
    pub repository_name: String,
    pub repository_owner: String,
}

#[derive(Debug, Deserialize)]
pub struct GetBoardRepositoryQueryParams {
    pub board_id: i64,
}
