use actix::{Actor, Addr};
use actix_web::{web};

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};
use crate::services::github_webhook_service::GithubWebhookService;
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::board_repository::{
    BoardRepositoryRepository, CreateBoardRepositoryMessage, GetBoardRepositoryByBoardIdMessage,
    UpdateRepositoryDataMessage,
};
use crate::state::State;
use crate::value_objects::gitrello_api::BoardPermissions;

pub struct BoardRepositoryService<'a> {
    state: &'a web::Data<State>,
    user: &'a User,
    actor: Addr<BoardRepositoryRepository>,
}

impl<'a> BoardRepositoryService<'a> {
    pub fn new(state: &'a web::Data<State>, user: &'a User) -> Result<Self, GITrelloError> {
        let connection = state.get_db_connection()?;
        let actor = BoardRepositoryRepository::new(connection).start();
        Ok(Self { actor, state, user })
    }

    pub async fn get_by_board_id(&self, board_id: i64) -> Result<BoardRepository, GITrelloError> {
        let permissions = self.get_permissions(board_id).await?;
        if !permissions.can_read {
            return Err(GITrelloError::PermissionDenied);
        }

        self.get(board_id).await
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

        let board_repository = self.get(board_id).await;
        return match board_repository {
            Ok(board_repository) => {
                let github_webhook_service = GithubWebhookService::new(self.state, self.user).await?;
                github_webhook_service
                    .update(&board_repository, repository_name, repository_owner)
                    .await?;

                self
                    .update_repository_data(board_repository, repository_name, repository_owner)
                    .await
                    .map(|board_repository| (board_repository, false))
            }
            Err(e) => {
                match e {
                    GITrelloError::NotFound {message: _ } => {
                        let board_repository = self
                            .create(board_id, repository_name, repository_owner)
                            .await?;

                        let github_webhook_service = GithubWebhookService::new(self.state, self.user).await?;
                        github_webhook_service
                            .create(&board_repository)
                            .await?;

                        Ok((board_repository, true))
                    },
                    _ => Err(e)
                }
            }
        };
    }

    async fn get_permissions(&self, board_id: i64) -> Result<BoardPermissions, GITrelloError> {
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

    async fn get(&self, board_id: i64) -> Result<BoardRepository, GITrelloError> {
        self.actor
            .send(GetBoardRepositoryByBoardIdMessage { board_id })
            .await
            .map_err(|source| GITrelloError::ActorError { source })?
    }

    async fn create(
        &self,
        board_id: i64,
        repository_name: &str,
        repository_owner: &str,
    ) -> Result<BoardRepository, GITrelloError>
    {
        self.actor
            .send(CreateBoardRepositoryMessage {
                data: NewBoardRepository {
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
