use actix::{Actor, Addr};
use actix_web::web::{BytesMut, Data};
use bytes::{Buf};
use futures;
use serde_json;

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::github_webhook::{GithubWebhook, NewGithubWebhook};
use crate::models::board_repository::BoardRepository;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::github_profile_service::GithubProfileService;
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::board_repository::{
    BoardRepositoryRepository, GetBoardRepositoryByRepositoryOwnerAndNameMessage,
};
use crate::services::repositories::github_webhook::{
    CreateGithubWebhookMessage, GetByRepositoryNameAndOwnerMessage, GithubWebhookRepository,
    UpdateWebhookIdMessage,
};
use crate::state::State;
use crate::value_objects::github_api::{Issue, Repository};
use crate::value_objects::request_data::webhook::{IssueWebhookRequest};

pub struct GithubWebhookService<'a> {
    state: &'a Data<State>,
    actor: Addr<GithubWebhookRepository>,
    github_api_client: GitHubAPIClient,
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

        let github_api_client = GitHubAPIClient::new(github_profile.access_token.as_str());

        Ok(Self { actor, state, github_api_client })
    }

    pub async fn create_or_update(
        &self,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_webhooks = self
            .get_by_repository_owner_and_name(repository_name, repository_owner)
            .await?;
        let existing_github_webhook = github_webhooks.first();

        if board_repository.repository_owner == repository_owner &&
                board_repository.repository_name == repository_name {
            // BoardRepository was just created

            match existing_github_webhook {
                Some(existing_github_webhook) => {
                    self.clone(existing_github_webhook, board_repository).await
                },
                None => {
                    self.create(board_repository, repository_name, repository_owner).await
                }
            }
        } else {
            // BoardRepository is going to be updated after webhooks are created

            let github_webhooks_for_old_repository = self
                .get_by_repository_owner_and_name(
                    board_repository.repository_name.as_str(),
                    board_repository.repository_owner.as_str(),
                )
                .await?;

            let count = github_webhooks_for_old_repository.len();
            let github_webhook_to_update = github_webhooks_for_old_repository
                .into_iter()
                .find(|github_webhook| github_webhook.board_repository_id == board_repository.id)
                .expect("can not be None");

            if count == 1 {
                // Webhook is being used only by current BoardRepository, so we should delete it

                self.github_api_client
                    .delete_webhook(
                        board_repository.repository_name.as_str(),
                        board_repository.repository_owner.as_str(),
                        github_webhook_to_update.webhook_id,
                    )
                    .await?;
            }

            match existing_github_webhook {
                Some(existing_github_webhook) => {
                    self
                        .update_from_existing_github_webhook(
                            github_webhook_to_update,
                            existing_github_webhook,
                        )
                        .await
                },
                None => {
                    self
                        .update(
                            github_webhook_to_update,
                            repository_name,
                            repository_owner,
                        )
                        .await
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
        let webhook = self.github_api_client
            .create_webhook(
                repository_name,
                repository_owner,
                self.state.webhook_url.as_str(),
            )
            .await?;

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
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let new_webhook = self.github_api_client
            .create_webhook(
                repository_name,
                repository_owner,
                self.state.webhook_url.as_str(),
            )
            .await?;

        self.actor
            .send(UpdateWebhookIdMessage { github_webhook, webhook_id: new_webhook.id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn clone(
        &self,
        github_webhook: &GithubWebhook,
        board_repository: &BoardRepository,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let data = NewGithubWebhook {
            webhook_id: github_webhook.webhook_id,
            board_repository_id: board_repository.id,
            url: self.state.webhook_url.clone(),
        };

        self.actor
            .send(CreateGithubWebhookMessage { data })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn update_from_existing_github_webhook(
        &self,
        github_webhook_to_update: GithubWebhook,
        existing_github_webhook: &GithubWebhook,
    ) -> Result<GithubWebhook, GITrelloError> {
        self.actor
            .send(
                UpdateWebhookIdMessage {
                    github_webhook: github_webhook_to_update,
                    webhook_id: existing_github_webhook.webhook_id,
                },
            )
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn get_by_repository_owner_and_name(
        &self,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<Vec<GithubWebhook>, GITrelloError>
    {
        self.actor
            .send(
                GetByRepositoryNameAndOwnerMessage {
                    repository_name: repository_name.to_string(),
                    repository_owner: repository_owner.to_string(),
                },
            )
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
