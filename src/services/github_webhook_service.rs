use actix::{Actor, Addr};
use actix_web::web::{BytesMut, Data};
use bytes::{Buf};
use futures;
use serde_json;

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::github_profile::GithubProfile;
use crate::models::github_webhook::{GithubWebhook, NewGithubWebhook};
use crate::models::board_repository::BoardRepository;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::github_profile_service::GithubProfileService;
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::board_repository::{
    BoardRepositoryRepository, GetBoardRepositoryByRepositoryOwnerAndNameMessage,
};
use crate::services::repositories::github_webhook::{
    CreateGithubWebhookMessage, GetByBoardRepositoryIdMessage, GithubWebhookRepository,
    UpdateWebhookIdMessage,
};
use crate::state::State;
use crate::value_objects::github_api::{Issue, Repository};
use crate::value_objects::request_data::webhook::{IssueWebhookRequest};

pub struct GithubWebhookService<'a> {
    state: &'a Data<State>,
    actor: Addr<GithubWebhookRepository>,
    github_profile: GithubProfile,
}

impl<'a> GithubWebhookService<'a> {
    pub async fn new(
        state: &'a Data<State>,
        user: &'a User,
    ) -> Result<GithubWebhookService<'a>, GITrelloError>
    {
        let connection = state.get_db_connection()?;
        let actor = GithubWebhookRepository::new(connection).start();

        let github_profile_service = GithubProfileService::new(state, user)?;
        let github_profile = github_profile_service
            .get_by_user_id(user.id.expect("already checked"))
            .await?;

        Ok(Self { actor, state, github_profile })
    }

    pub async fn create_or_update(
        &self,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_webhook = self
            .get_by_board_repository_id(board_repository.id)
            .await;

        match github_webhook {
            Ok(github_webhook) => {
                self.update(github_webhook, board_repository, repository_name, repository_owner).await
            },
            Err(e) => {
                match e {
                    GITrelloError::NotFound { message: _} => {
                        self.create(board_repository, repository_name, repository_owner).await
                    },
                    _ => Err(e)
                }
            }
        }
    }

    async fn create(
        &self,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());
        let webhook = github_api_client
            .create_webhook(
                repository_name,
                repository_owner,
                self.state.webhook_url.as_str(),
            )
            .await?;

        // TODO
        // We can not create two webhooks with same config for one repository,
        //
        // So we need to create new GithubWebhook with the same `webhook_id` without actually
        // calling Github API.
        //
        // We also must not delete webhook when updating a single BoardRepository, if this webhook
        // is being used by several BoardRepositories
        let data = NewGithubWebhook {
            webhook_id: webhook.id,
            board_repository_id: board_repository.id,
            url: self.state.webhook_url.clone(),
        };

        self.actor
            .send(CreateGithubWebhookMessage { data })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn update(
        &self,
        github_webhook: GithubWebhook,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());

        // TODO
        // We can not create two webhooks with same config for one repository,
        //
        // So we need to create new GithubWebhook with the same `webhook_id` without actually
        // calling Github API.
        //
        // We also must not delete webhook when updating a single BoardRepository, if this webhook
        // is being used by several BoardRepositories
        let delete_old_webhook_future = github_api_client
            .delete_webhook(
                board_repository.repository_name.as_str(),
                board_repository.repository_owner.as_str(),
                github_webhook.webhook_id,
            );
        let create_new_webhook_future = github_api_client
            .create_webhook(
                repository_name,
                repository_owner,
                self.state.webhook_url.as_str(),
            );

        // Note: there is no need to return Err if old webhook was not deleted for some reason
        let results = futures::join!(delete_old_webhook_future, create_new_webhook_future);
        let new_webhook = results.1?;

        self.actor
            .send(UpdateWebhookIdMessage { github_webhook, webhook_id: new_webhook.id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn get_by_board_repository_id(
        &self,
        board_repository_id: i32,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        self.actor
            .send(GetByBoardRepositoryIdMessage { board_repository_id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }
}

pub struct GithubWebhookProcessingService<'a> {
    state: &'a Data<State>,
}

impl<'a> GithubWebhookProcessingService<'a> {
    pub fn new(state: &'a Data<State>) -> Self {
        Self { state }
    }

    pub async fn process(&self, event_type: &str, request_body: BytesMut) -> Result<(), GITrelloError>{
        match event_type {
            "issues" => {
                let request_json = serde_json::from_slice::<IssueWebhookRequest>(
                        request_body.bytes(),
                    )
                    .map_err(|_| GITrelloError::InternalError)?;

                match request_json.action.as_str() {
                    "opened" => {
                        self.process_issue_opened(&request_json.issue, &request_json.repository).await?;
                    },
                    _ => ()
                }
            },
            _ => ()
        }

        Ok(())
    }

    pub async fn process_issue_opened(&self, issue: &Issue, repository: &Repository) -> Result<(), GITrelloError>{
        let parts = repository.split_full_name();

        let connection = self.state.get_db_connection()?;
        let actor = BoardRepositoryRepository::new(connection).start();

        let board_repositories: Vec<BoardRepository> = actor
            .send(GetBoardRepositoryByRepositoryOwnerAndNameMessage {
                repository_owner: parts.0.to_string(),
                repository_name: parts.1.to_string(),
            })
            .await
            .map_err(|source| GITrelloError::ActorError { source })??;

        let gitrello_api_client = GITRelloAPIClient::with_access_token(
            &self.state.gitrello_url,
            &self.state.gitrello_access_token,
        );

        let mut create_ticket_futures = Vec::new();
        for board_repository in board_repositories.iter() {
            create_ticket_futures.push(
                gitrello_api_client.create_ticket(
                    board_repository.board_id,
                    issue.title.as_str(),
                    issue.body.as_str(),
                ),
            );
        }

        futures::future::join_all(create_ticket_futures).await;

        Ok(())
    }
}
