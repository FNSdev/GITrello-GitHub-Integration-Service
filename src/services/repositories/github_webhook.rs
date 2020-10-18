use actix::{Actor, Context, Handler, Message};
use diesel::{insert_into, update, result::Error, BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};

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

    pub fn create(&self, data: &NewGithubWebhook) -> Result<GithubWebhook, GITrelloError>
    {
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
    ) -> Result<GithubWebhook, GITrelloError>
    {
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

    pub fn get_by_repository_name_and_owner(
        &self,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<Vec<GithubWebhook>, GITrelloError>
    {
        use crate::schema::github_webhook;
        use crate::schema::board_repository;

        github_webhook::table
            .select(github_webhook::all_columns)
            .inner_join(board_repository::table)
            .filter(
                board_repository::repository_name.eq(repository_name)
                    .and(board_repository::repository_owner.eq(repository_owner)),
            )
            .load::<GithubWebhook>(&self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!(
                                "github_webhook for repository {}/{} does not exist",
                                repository_name,
                                repository_owner,
                            ),
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
#[rtype(result = "Result<Vec<GithubWebhook>, GITrelloError>")]
pub struct GetByRepositoryNameAndOwnerMessage {
    pub repository_name: String,
    pub repository_owner: String,
}

impl Handler<GetByRepositoryNameAndOwnerMessage> for GithubWebhookRepository {
    type Result = Result<Vec<GithubWebhook>, GITrelloError>;

    fn handle(
        &mut self,
        msg: GetByRepositoryNameAndOwnerMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.get_by_repository_name_and_owner(
            msg.repository_name.as_str(),
            msg.repository_owner.as_str(),
        )
    }
}
