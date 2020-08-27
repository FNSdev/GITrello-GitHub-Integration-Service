use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct APIError {
    pub error_message: String,
    pub error_code: i16,
}

#[derive(Serialize, Debug)]
pub struct GetBoardPermissionsRequest {
    pub user_id: i64,
    pub board_id: i64,
}

#[derive(Deserialize)]
pub struct BoardPermissions {
    pub can_read: bool,
    pub can_mutate: bool
}