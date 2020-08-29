use actix::{Actor, Context, Handler, Message};
use diesel::{
    insert_into, update, result::DatabaseErrorKind, result::Error, RunQueryDsl, QueryDsl,
    ExpressionMethods,
};

use crate::errors::GITrelloError;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};
use crate::state::DbConnection;

pub struct BoardRepositoryRepository {
    connection: DbConnection,
}

impl BoardRepositoryRepository {
    pub fn new(connection: DbConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, data: &NewBoardRepository) -> Result<BoardRepository, GITrelloError> {
        use crate::schema::board_repository::dsl::*;

        insert_into(board_repository)
            .values(data)
            .get_result(&self.connection)
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
            .first::<BoardRepository>(&self.connection)
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

    pub fn update_repository_data(
        &self,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<BoardRepository, GITrelloError>
    {
        use crate::schema::board_repository::{
            repository_name as repository_name_column,
            repository_owner as repository_owner_column,
        };

        update(board_repository)
            .set((
                repository_name_column.eq(repository_name),
                repository_owner_column.eq(repository_owner),
            ))
            .get_result::<BoardRepository>(&self.connection)
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

impl Actor for BoardRepositoryRepository {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<BoardRepository, GITrelloError>")]
pub struct CreateBoardRepositoryMessage {
    pub data: NewBoardRepository,
}

impl Handler<CreateBoardRepositoryMessage> for BoardRepositoryRepository {
    type Result = Result<BoardRepository, GITrelloError>;

    fn handle(
        &mut self,
        msg: CreateBoardRepositoryMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.create(&msg.data)
    }
}

#[derive(Message)]
#[rtype(result = "Result<BoardRepository, GITrelloError>")]
pub struct GetBoardRepositoryByBoardIdMessage {
    pub board_id: i64,
}

impl Handler<GetBoardRepositoryByBoardIdMessage> for BoardRepositoryRepository {
    type Result = Result<BoardRepository, GITrelloError>;

    fn handle(
        &mut self,
        msg: GetBoardRepositoryByBoardIdMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.get_by_board_id(msg.board_id)
    }
}

#[derive(Message)]
#[rtype(result = "Result<BoardRepository, GITrelloError>")]
pub struct UpdateRepositoryDataMessage {
    pub board_repository: BoardRepository,
    pub repository_name: String,
    pub repository_owner: String,
}

impl Handler<UpdateRepositoryDataMessage> for BoardRepositoryRepository {
    type Result = Result<BoardRepository, GITrelloError>;

    fn handle(
        &mut self,
        msg: UpdateRepositoryDataMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.update_repository_data(
            &msg.board_repository,
            msg.repository_name.as_str(),
            msg.repository_owner.as_str(),
        )
    }
}
