use actix::Actor;
use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::models::board_repository::{BoardRepository, NewBoardRepository};
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::board_repository::{
    BoardRepositoryRepository, GetBoardRepositoryByBoardIdMessage, CreateBoardRepositoryMessage,
    UpdateRepositoryIdMessage,
};
use crate::state::State;
use crate::value_objects::request_data::board_repository::{
    GetBoardRepositoryQueryParams, NewBoardRepositoryRequest,
};
use crate::value_objects::response_data::board_repository::BoardRepositoryResponse;
use crate::errors::GITrelloError;

#[put("/api/v1/board-repositories")]
pub async fn create_board_repository(
    req: HttpRequest,
    json: web::Json<NewBoardRepositoryRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let gitrello_api_client = GITRelloAPIClient::new(state.gitrello_url.as_str());
    let board_permissions = gitrello_api_client
        .get_board_permissions(
            user.id.expect("is_authenticated() must be checked earlier"),
            json.board_id,
        )
        .await;

    match board_permissions {
        Ok(board_permissions) => {
            if !board_permissions.can_mutate {
                return Err(GITrelloError::PermissionDenied);
            }
        },
        Err(_) => {
            return Err(GITrelloError::InternalError);
        }
    }

    let connection = state.get_db_connection()?;
    let board_repository_actor = BoardRepositoryRepository::new(connection).start();
    let board_repository: Result<BoardRepository, GITrelloError> = board_repository_actor
        .send(GetBoardRepositoryByBoardIdMessage { board_id: json.board_id })
        .await
        .map_err(|source| GITrelloError::ActorError { source })?;

    return match board_repository {
        Ok(board_repository) => {
            // todo update webhooks

            let message = UpdateRepositoryIdMessage {
                board_repository,
                repository_id: json.repository_id,
            };
            let board_repository: BoardRepository = board_repository_actor
                .send(message)
                .await
                .map_err(|source| GITrelloError::ActorError { source })??;

            Ok(HttpResponse::Ok().json(BoardRepositoryResponse {
                id: board_repository.id,
                board_id: board_repository.board_id.to_string(),
                repository_id: board_repository.repository_id.to_string(),
            }))
        }
        Err(e) => {
            match e {
                GITrelloError::NotFound {message: _ } => {
                    // todo create webhooks

                    let board_repository: BoardRepository = board_repository_actor
                        .send(CreateBoardRepositoryMessage {
                            data: NewBoardRepository {
                                board_id: json.board_id,
                                repository_id: json.repository_id,
                            },
                        })
                        .await
                        .map_err(|source| GITrelloError::ActorError { source })??;

                    Ok(HttpResponse::Created().json(BoardRepositoryResponse {
                        id: board_repository.id,
                        board_id: board_repository.board_id.to_string(),
                        repository_id: board_repository.repository_id.to_string(),
                    }))
                },
                _ => Err(e)
            }
        }
    }
}

#[get("/api/v1/board-repository")]
pub async fn get_board_repository(
    req: HttpRequest,
    web::Query(query_params): web::Query<GetBoardRepositoryQueryParams>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let gitrello_api_client = GITRelloAPIClient::new(state.gitrello_url.as_str());
    let board_permissions = gitrello_api_client
        .get_board_permissions(
            user.id.expect("is_authenticated() must be checked earlier"),
            query_params.board_id,
        )
        .await;

    if board_permissions.is_err() {
        return Err(GITrelloError::InternalError);
    }

    if !board_permissions.expect("already checked").can_read {
        return Err(GITrelloError::PermissionDenied);
    }

    let connection = state.get_db_connection()?;
    let board_repository_actor = BoardRepositoryRepository::new(connection).start();
    let board_repository: BoardRepository = board_repository_actor
        .send(GetBoardRepositoryByBoardIdMessage { board_id: query_params.board_id })
        .await
        .map_err(|source| GITrelloError::ActorError { source })??;

    Ok(HttpResponse::Ok().json(BoardRepositoryResponse {
        id: board_repository.id,
        board_id: board_repository.board_id.to_string(),
        repository_id: board_repository.repository_id.to_string(),
    }))
}
