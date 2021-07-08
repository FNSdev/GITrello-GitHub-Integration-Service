use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::services::board_repository_service::BoardRepositoryService;
use crate::state::State;
use crate::value_objects::request_data::board_repository::{
    GetBoardRepositoryQueryParams, NewBoardRepositoryRequest,
};
use crate::value_objects::response_data::board_repository::BoardRepositoryResponse;
use crate::errors::GITrelloError;

#[put("/api/v1/board-repositories")]
pub async fn create_or_update_board_repository(
    req: HttpRequest,
    json: web::Json<NewBoardRepositoryRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let board_repository_service = BoardRepositoryService::new(&state, &user)?;
    let result = board_repository_service
        .create_or_update(
            json.board_id,
            json.repository_name.as_str(),
            json.repository_owner.as_str(),
        )
        .await;

    return match result {
        Ok(board_repository_upsert_result) => {
            let response_data = BoardRepositoryResponse {
                id: board_repository_upsert_result.0.id,
                board_id: board_repository_upsert_result.0.board_id.to_string(),
                repository_name: board_repository_upsert_result.0.repository_name,
                repository_owner: board_repository_upsert_result.0.repository_owner,
            };

            match board_repository_upsert_result.1 {
                true => Ok(HttpResponse::Created().json(response_data)),
                _ => Ok(HttpResponse::Ok().json(response_data))
            }
        },
        Err(e) => Err(e)
    }
}

#[delete("/api/v1/board-repositories/{id}")]
pub async fn delete_board_repository(
    req: HttpRequest,
    web::Path((id, )): web::Path<(i32, )>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError> {
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let board_repository_service = BoardRepositoryService::new(&state, &user)?;
    board_repository_service.delete(id).await?;

    Ok(HttpResponse::NoContent().finish())
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

    let board_repository_service = BoardRepositoryService::new(&state, &user)?;
    let board_repository = board_repository_service
        .get_by_board_id(query_params.board_id)
        .await?;

    let response_data = BoardRepositoryResponse {
        id: board_repository.id,
        board_id: board_repository.board_id.to_string(),
        repository_name: board_repository.repository_name,
        repository_owner: board_repository.repository_owner,
    };

    Ok(HttpResponse::Ok().json(response_data))
}
