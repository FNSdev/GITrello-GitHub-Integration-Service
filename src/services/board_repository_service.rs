use actix::{Actor, Addr};
use actix_web::{web::Data, };

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};
use crate::models::github_webhook::GithubWebhook;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::github_profile_service::GithubProfileService;
use crate::services::github_webhook_service::GithubWebhookService;
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::github_webhook::{
    GetByRepositoryNameAndOwnerMessage, GithubWebhookRepository,
};
use crate::services::repositories::board_repository::{
    BoardRepositoryRepository, CreateBoardRepositoryMessage, DeleteBoardRepositoryMessage,
    GetBoardRepositoryByBoardIdMessage, GetBoardRepositoryByIdMessage, UpdateRepositoryDataMessage,
};
use crate::state::State;
use crate::value_objects::gitrello_api::Permissions;

pub struct BoardRepositoryService<'a> {
    state: &'a Data<State>,
    user: &'a User,
    actor: Addr<BoardRepositoryRepository>,
}

impl<'a> BoardRepositoryService<'a> {
    pub fn new(state: &'a Data<State>, user: &'a User) -> Result<Self, GITrelloError> {
        let connection = state.get_db_connection()?;
        let actor = BoardRepositoryRepository::new(connection).start();
        Ok(Self { actor, state, user })
    }

    pub async fn get_by_board_id(&self, board_id: i64) -> Result<BoardRepository, GITrelloError> {
        let permissions = self.get_permissions(board_id).await?;
        if !permissions.can_read {
            return Err(GITrelloError::PermissionDenied);
        }

        self._get_by_board_id(board_id).await
    }

    pub async fn create_or_update(
        &self,
        board_id: i64,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<(BoardRepository, bool), GITrelloError>
    {
        let permissions = self.get_permissions(board_id).await?;
        if !permissions.can_mutate {
            return Err(GITrelloError::PermissionDenied);
        }

        let board_repository = self._get_by_board_id(board_id).await;
        return match board_repository {
            Ok(board_repository) => {
                if board_repository.repository_name == repository_name &&
                        board_repository.repository_owner == repository_owner {
                    return Ok((board_repository, false));
                }

                let github_webhook_service = GithubWebhookService::new(self.state, self.user).await?;
                github_webhook_service
                    .create_or_update(&board_repository, repository_name, repository_owner)
                    .await?;

                self
                    .update_repository_data(board_repository, repository_name, repository_owner)
                    .await
                    .map(|board_repository| (board_repository, false))
            }
            Err(e) => {
                match e {
                    GITrelloError::NotFound {message: _ } => {
                        let github_profile_service = GithubProfileService::new(self.state, self.user)?;
                        let github_profile = github_profile_service
                            .get_by_user_id(self.user.id.expect("already checked"))
                            .await?;

                        let board_repository = self
                            .create(github_profile.id, board_id, repository_name, repository_owner)
                            .await?;

                        let github_webhook_service = GithubWebhookService::new(self.state, self.user).await?;
                        github_webhook_service
                            .create_or_update(&board_repository, repository_name, repository_owner)
                            .await?;

                        Ok((board_repository, true))
                    },
                    _ => Err(e)
                }
            }
        };
    }

    pub async fn delete(&self, id: i32) -> Result<(), GITrelloError> {
        let board_repository = self._get_by_id(id).await?;

        let permissions = self.get_permissions(board_repository.board_id).await?;
        if !permissions.can_mutate {
            return Err(GITrelloError::PermissionDenied);
        }

        let connection = self.state.get_db_connection()?;
        let github_webhook_repository_actor = GithubWebhookRepository::new(connection).start();

        let github_webhooks: Vec<GithubWebhook> = github_webhook_repository_actor
            .send(
                GetByRepositoryNameAndOwnerMessage {
                    repository_name: board_repository.repository_name.clone(),
                    repository_owner: board_repository.repository_owner.clone(),
                },
            )
            .await
            .map_err(|source| GITrelloError::ActorError { source })??;

        if github_webhooks.len() == 1 {
            let github_profile_service = GithubProfileService::new(self.state, self.user)?;
            let github_profile = github_profile_service
                .get_by_user_id(self.user.id.expect("already checked"))
                .await?;

            let api_client = GitHubAPIClient::new(github_profile.access_token.as_str());
            api_client
                .delete_webhook(
                    board_repository.repository_name.as_str(),
                    board_repository.repository_owner.as_str(),
                    github_webhooks.first().expect("already checked").webhook_id,
                )
                .await?;
        }

        self.actor
            .send(DeleteBoardRepositoryMessage { id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn get_permissions(&self, board_id: i64) -> Result<Permissions, GITrelloError> {
        let gitrello_api_client = GITRelloAPIClient::new(
            self.state.gitrello_url.as_str(),
        );

        gitrello_api_client
            .get_board_permissions(
                self.user.id.expect("is_authenticated() must be checked earlier"),
                board_id,
            )
            .await
    }

    async fn _get_by_board_id(&self, board_id: i64) -> Result<BoardRepository, GITrelloError> {
        self.actor
            .send(GetBoardRepositoryByBoardIdMessage { board_id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn _get_by_id(&self, id: i32) -> Result<BoardRepository, GITrelloError> {
        self.actor
            .send(GetBoardRepositoryByIdMessage { id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn create(
        &self,
        github_profile_id: i32,
        board_id: i64,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<BoardRepository, GITrelloError>
    {
        self.actor
            .send(CreateBoardRepositoryMessage {
                data: NewBoardRepository {
                    github_profile_id,
                    board_id,
                    repository_name: repository_name.to_string(),
                    repository_owner: repository_owner.to_string(),
                },
            })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn update_repository_data(
        &self,
        board_repository: BoardRepository,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<BoardRepository, GITrelloError>
    {
        self.actor
            .send(UpdateRepositoryDataMessage {
                board_repository,
                repository_name: repository_name.to_string(),
                repository_owner: repository_owner.to_string(),
            })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }
}
