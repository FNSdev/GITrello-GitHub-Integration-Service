use serde::{Deserialize, Serialize};

use crate::schema::board_repository;

#[derive(Debug, Clone, Serialize, Queryable)]
pub struct BoardRepository {
    pub id: i32,
    pub board_id: i64,
    pub repository_id: i64,
    pub github_profile_id: i32,
}

#[table_name = "board_repository"]
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
pub struct NewBoardRepository {
    pub board_id: i64,
    pub repository_id: i64,
    pub github_profile_id: i32,
}
