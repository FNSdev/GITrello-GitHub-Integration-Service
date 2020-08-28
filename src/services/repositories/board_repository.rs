use diesel::{
    insert_into, update, result::DatabaseErrorKind, result::Error, RunQueryDsl, QueryDsl,
    ExpressionMethods,
};

use crate::errors::GITrelloError;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};
use crate::state::DbConnection;

pub struct BoardRepositoryRepository<'a> {
    connection: &'a DbConnection,
}

impl <'a> BoardRepositoryRepository<'a> {
    pub fn new(connection: &'a DbConnection) -> Self {
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

    pub fn update_repository_id(
        &self,
        board_repository: &BoardRepository,
        repository_id: i64,
    ) -> Result<BoardRepository, GITrelloError>
    {
        use crate::schema::board_repository::{repository_id as repository_id_column};

        update(board_repository)
            .set(repository_id_column.eq(repository_id))
            .get_result::<BoardRepository>(self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("board_repository {} does not exist", board_repository.id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }
}
