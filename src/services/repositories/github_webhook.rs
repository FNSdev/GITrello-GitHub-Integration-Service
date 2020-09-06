use actix::{Actor, Context, Handler, Message};
use diesel::{insert_into, pg::upsert::excluded, ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::errors::GITrelloError;
use crate::models::github_webhook::{GithubWebhook, NewGithubWebhook};
use crate::state::DbConnection;

pub struct GithubWebhookRepository {
    connection: DbConnection,
}

impl GithubWebhookRepository {
    pub fn new(connection: DbConnection) -> Self {
        Self { connection }
    }

    pub fn create_or_update(&self, data: &NewGithubWebhook) -> Result<GithubWebhook, GITrelloError> {
        use crate::schema::github_webhook::dsl::*;

        insert_into(github_webhook)
            .values(data)
            .on_conflict(board_repository_id)
            .do_update()
            .set(webhook_id.eq(excluded(webhook_id)))
            .get_result(&self.connection)
            .map_err(|source| GITrelloError::DieselError { source })
    }

    pub fn get_by_board_repository_id(
        &self,
        board_repository_id: i32,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        use crate::schema::github_webhook::{
            table, board_repository_id as board_repository_id_column,
        };

        table
            .filter(board_repository_id_column.eq(board_repository_id))
            .first::<GithubWebhook>(&self.connection)
            .map_err(|source| GITrelloError::DieselError { source })
    }
}

impl Actor for GithubWebhookRepository {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<GithubWebhook, GITrelloError>")]
pub struct CreateOrUpdateGithubWebhookMessage {
    pub data: NewGithubWebhook,
}

impl Handler<CreateOrUpdateGithubWebhookMessage> for GithubWebhookRepository {
    type Result = Result<GithubWebhook, GITrelloError>;

    fn handle(
        &mut self,
        msg: CreateOrUpdateGithubWebhookMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.create_or_update(&msg.data)
    }
}

#[derive(Message)]
#[rtype(result = "Result<GithubWebhook, GITrelloError>")]
pub struct GetByBoardRepositoryIdMessage {
    pub board_repository_id: i32,
}

impl Handler<GetByBoardRepositoryIdMessage> for GithubWebhookRepository {
    type Result = Result<GithubWebhook, GITrelloError>;

    fn handle(
        &mut self,
        msg: GetByBoardRepositoryIdMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.get_by_board_repository_id(msg.board_repository_id)
    }
}
