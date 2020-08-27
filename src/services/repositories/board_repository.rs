use diesel::{
    insert_into, result::DatabaseErrorKind, result::Error, RunQueryDsl, QueryDsl, ExpressionMethods
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

use crate::errors::GITrelloError;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};

pub struct BoardRepositoryRepository<'a> {
    connection: &'a PooledConnection<ConnectionManager<PgConnection>>,
}

impl <'a> BoardRepositoryRepository<'a> {
    pub fn new(connection: &'a PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { connection }
    }

    pub fn create(&self, data: &NewBoardRepository) -> Result<BoardRepository, GITrelloError> {
        use crate::schema::board_repository::dsl::*;

        insert_into(board_repository)
            .values(data)
            .get_result(self.connection)
            .map_err(|source| {
                match source {
                    Error::DatabaseError(DatabaseErrorKind::UniqueViolation, error_info) => {
                        GITrelloError::AlreadyExists { message: String::from(error_info.message()) }
                    },
                    Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        GITrelloError::NotFound {
                            message: format!(
                                "github_profile with id {} does not exist",
                                data.github_profile_id,
                            ),
                        }
                    }
                    _ => GITrelloError::DieselError { source }
                }
            })
    }

    pub fn get_by_board_id(&self, board_id: i64) -> Result<BoardRepository, GITrelloError> {
        use crate::schema::board_repository::{table, board_id as board_id_column};

        table
            .filter(board_id_column.eq(board_id))
            .first::<BoardRepository>(self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("board_repository for board {} does not exist", board_id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }
}
