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
    CreateOrUpdateGithubWebhookMessage, GetByBoardRepositoryIdMessage, GithubWebhookRepository,
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

    pub async fn create(
        &self,
        board_repository: &BoardRepository,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());
        let webhook = github_api_client
            .create_webhook(
                board_repository.repository_name.as_str(),
                board_repository.repository_owner.as_str(),
                self.state.webhook_url.as_str(),
            )
            .await?;

        self
            .create_or_update(webhook.id, board_repository.id, self.state.webhook_url.as_str())
            .await
    }

    pub async fn update(
        &self,
        board_repository: &BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(self.github_profile.access_token.as_str());
        let github_webhook = self
            .get_by_board_repository_id(board_repository.id)
            .await?;

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

        let results = futures::try_join!(delete_old_webhook_future, create_new_webhook_future)?;
        let new_webhook = results.1;

        self
            .create_or_update(new_webhook.id, board_repository.id, self.state.webhook_url.as_str())
            .await
    }

    async fn create_or_update(
        &self,
        webhook_id: i64,
        board_repository_id: i32,
        url: &str,
    ) -> Result<GithubWebhook, GITrelloError>
    {
        let data = NewGithubWebhook {webhook_id, board_repository_id, url: url.to_string() };

        self.actor
            .send(CreateOrUpdateGithubWebhookMessage { data })
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
