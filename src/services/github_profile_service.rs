use actix::{Actor, Addr};
use actix_web::{web};

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::github_profile::{GithubProfile, NewGithubProfile};
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::{
    GetGithubProfileByUserIdMessage, GithubProfileRepository, CreateGithubProfileMessage,
};
use crate::state::State;

pub struct GithubProfileService<'a> {
    user: &'a User,
    actor: Addr<GithubProfileRepository>,
}

impl<'a> GithubProfileService<'a> {
    pub fn new(state: &'a web::Data<State>, user: &'a User) -> Result<Self, GITrelloError> {
        let connection = state.get_db_connection()?;
        let actor = GithubProfileRepository::new(connection).start();
        Ok(Self { actor, user })
    }

    pub async fn get_by_user_id(&self, user_id: i64) -> Result<GithubProfile, GITrelloError> {
        self.actor
            .send(GetGithubProfileByUserIdMessage { user_id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    pub async fn create(&self, access_token: &str) -> Result<GithubProfile, GITrelloError>
    {
        let github_api_client = GitHubAPIClient::new(access_token);
        let github_user = github_api_client.get_user().await?;

        self.actor
            .send(CreateGithubProfileMessage {
                data: NewGithubProfile {
                    user_id: self.user.id.expect("is_authenticated() must be checked earlier"),
                    github_user_id: github_user.id,
                    github_login: github_user.login,
                    access_token: access_token.to_string(),
                },
            })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }
}
