use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::errors::ToGITrelloError;
use crate::models::board_repository::NewBoardRepository;
use crate::services::gitrello_api_client::GITRelloAPIClient;
use crate::services::repositories::board_repository::BoardRepositoryRepository;
use crate::state::State;
use crate::value_objects::request_data::board_repository::NewBoardRepositoryRequest;
use crate::errors::GITrelloError;

#[post("/api/v1/board-repositories")]
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

    let connection = state.db_pool
        .get()
        .map_err(|source| GITrelloError::R2D2Error { source })?;

    let gitrello_api_client = GITRelloAPIClient::new(state.gitrello_url.as_str());
    let board_permissions = gitrello_api_client
        .get_board_permissions(
            user.id.expect("is_authenticated() must be checked earlier"),
            json.board_id,
        )
        .await;

    if board_permissions.is_err() {
        return Err(GITrelloError::InternalError);
    }

    if !board_permissions.expect("already checked").can_mutate {
        return Err(GITrelloError::PermissionDenied);
    }

    let data = NewBoardRepository {
        board_id: json.board_id,
        repository_id: json.repository_id,
    };

    let board_repository = web::
        block(
            move || {
                let repository = BoardRepositoryRepository::new(&connection);
                repository.create(&data)
            }
        )
        .await
        .map_err(|e| e.move_to_gitrello_error())?;

    Ok(HttpResponse::Created().json(board_repository))
}
