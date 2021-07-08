use actix::{Actor, Addr};
use actix_web::{web};
use futures::future;

use crate::errors::GITrelloError;
use crate::models::github_profile::{GithubProfile, NewGithubProfile};
use crate::models::github_webhook::{GithubWebhookWithRepositoryInfo};
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::{
    GetGithubProfileByGithubUserIdMessage, GetGithubProfileByUserIdMessage, GithubProfileRepository,
    CreateGithubProfileMessage, DeleteGithubProfileMessage,
};
use crate::services::repositories::github_webhook::{
    GetDistinctWebhooksByGithubProfileIdMessage, GithubWebhookRepository,
};
use crate::state::State;

pub struct GithubProfileService<'a> {
    state: &'a State,
    actor: Addr<GithubProfileRepository>,
}

impl<'a> GithubProfileService<'a> {
    pub fn new(state: &'a web::Data<State>) -> Result<Self, GITrelloError> {
        let connection = state.get_db_connection()?;
        let actor = GithubProfileRepository::new(connection).start();
        Ok(Self { actor, state })
    }

    pub async fn get_by_user_id(&self, user_id: i64) -> Result<GithubProfile, GITrelloError> {
        self.actor
            .send(GetGithubProfileByUserIdMessage { user_id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    pub async fn create(&self, user_id: i64, access_token: &str) -> Result<GithubProfile, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(access_token);
        let github_user = github_api_client.get_user().await?;

        let existing_github_profile = self.actor
            .send(GetGithubProfileByGithubUserIdMessage { github_user_id: github_user.id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?;

        match existing_github_profile {
            Ok(_) => {
                Err(
                    GITrelloError::AlreadyExists {
                        message: String::from("This GitHub account is being used by another GITrello user"),
                    },
                )
            },
            Err(e) => {
                match e {
                    GITrelloError::NotFound { message: _ } => {
                        self.actor
                            .send(CreateGithubProfileMessage {
                                data: NewGithubProfile {
                                    user_id,
                                    github_user_id: github_user.id,
                                    github_login: github_user.login,
                                    access_token: access_token.to_string(),
                                },
                            })
                            .await
                            .map_err(|source| GITrelloError::ActorError { source })?
                    },
                    _ => Err(e)
                }
            }
        }
    }

    pub async fn delete(&self, user_id: i64) -> Result<(), GITrelloError> {
        let github_profile = self
            .get_by_user_id(user_id)
            .await?;

        let connection = self.state.get_db_connection()?;
        let actor = GithubWebhookRepository::new(connection).start();

        let webhooks: Vec<GithubWebhookWithRepositoryInfo> = actor
            .send(GetDistinctWebhooksByGithubProfileIdMessage { github_profile_id: github_profile.id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })??;

        let github_api_client = GitHubAPIClient::new(github_profile.access_token.as_str());

        let mut delete_webhook_futures = Vec::new();
        for webhook_info in webhooks.iter() {
            delete_webhook_futures.push(
                github_api_client.delete_webhook(
                    webhook_info.board_repository_name.as_str(),
                    webhook_info.board_repository_owner.as_str(),
                    webhook_info.webhook_id,
                ),
            );
        }

        future::join_all(delete_webhook_futures).await;

        self.actor
            .send(DeleteGithubProfileMessage { id: github_profile.id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }
}
