use actix::{Actor, Addr};
use actix_web::{web};
use futures;

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::github_profile::GithubProfile;
use crate::models::github_webhook::{GithubWebhook, NewGithubWebhook};
use crate::models::board_repository::BoardRepository;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::github_profile_service::GithubProfileService;
use crate::services::repositories::github_webhook::{
    CreateGithubWebhookMessage, GetByBoardRepositoryIdMessage, GithubWebhookRepository,
    UpdateWebhookIdMessage,
};
use crate::state::State;

pub struct GithubWebhookService<'a> {
    state: &'a web::Data<State>,
    actor: Addr<GithubWebhookRepository>,
    github_profile: GithubProfile,
}

impl<'a> GithubWebhookService<'a> {
    pub async fn new(
        state: &'a web::Data<State>,
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
    ) -> Result<GithubWebhook, GITrelloError> {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());
        let webhook = github_api_client
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
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError> {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());

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
