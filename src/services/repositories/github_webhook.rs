use actix::{Actor, Context, Handler, Message};
use diesel::{insert_into, update, result::Error, ExpressionMethods, QueryDsl, RunQueryDsl};

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

    pub fn create(&self, data: &NewGithubWebhook) -> Result<GithubWebhook, GITrelloError> {
        use crate::schema::github_webhook::dsl::*;

        insert_into(github_webhook)
            .values(data)
            .get_result(&self.connection)
            .map_err(|source| GITrelloError::DieselError { source })
    }

    pub fn update_webhook_id(
        &self,
        github_webhook: &GithubWebhook,
        webhook_id: i64,
    ) -> Result<GithubWebhook, GITrelloError> {
        use crate::schema::github_webhook::{webhook_id as webhook_id_column};

        update(github_webhook)
            .set(webhook_id_column.eq(webhook_id))
            .get_result::<GithubWebhook>(&self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("github_webhook {} does not exist", github_webhook.id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
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
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("github_webhook for board_repository {} does not exist", board_repository_id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }
}

impl Actor for GithubWebhookRepository {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<GithubWebhook, GITrelloError>")]
pub struct CreateGithubWebhookMessage {
    pub data: NewGithubWebhook,
}

impl Handler<CreateGithubWebhookMessage> for GithubWebhookRepository {
    type Result = Result<GithubWebhook, GITrelloError>;

    fn handle(
        &mut self,
        msg: CreateGithubWebhookMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.create(&msg.data)
    }
}

#[derive(Message)]
#[rtype(result = "Result<GithubWebhook, GITrelloError>")]
pub struct UpdateWebhookIdMessage {
    pub github_webhook: GithubWebhook,
    pub webhook_id: i64,
}

impl Handler<UpdateWebhookIdMessage> for GithubWebhookRepository {
    type Result = Result<GithubWebhook, GITrelloError>;

    fn handle(
        &mut self,
        msg: UpdateWebhookIdMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.update_webhook_id(&msg.github_webhook, msg.webhook_id)
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
