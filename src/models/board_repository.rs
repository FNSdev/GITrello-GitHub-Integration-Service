use serde::{Deserialize, Serialize};

use crate::schema::board_repository;

#[table_name = "board_repository"]
#[derive(Debug, Clone, Identifiable, Serialize, Queryable)]
pub struct BoardRepository {
    pub id: i32,
    pub github_profile_id: i32,
    pub board_id: i64,
    pub repository_name: String,
    pub repository_owner: String,
}

#[table_name = "board_repository"]
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
pub struct NewBoardRepository {
    pub github_profile_id: i32,
    pub board_id: i64,
    pub repository_name: String,
    pub repository_owner: String,
}
