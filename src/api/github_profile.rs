use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::services::github_profile_service::GithubProfileService;
use crate::state::State;
use crate::value_objects::request_data::github_profile::NewGithubProfileRequest;
use crate::value_objects::response_data::github_profile::GithubProfileResponse;
use crate::errors::GITrelloError;

#[post("/api/v1/github-profiles")]
pub async fn create_github_profile(
    req: HttpRequest,
    json: web::Json<NewGithubProfileRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let github_profile_service = GithubProfileService::new(&state)?;
    let github_profile = github_profile_service
        .create(user.id.expect("already checked"), json.access_token.as_str())
        .await?;

    let response_data = GithubProfileResponse {
        id: github_profile.id,
        user_id: github_profile.user_id.to_string(),
        github_user_id: github_profile.github_user_id.to_string(),
        github_login: github_profile.github_login
    };

    Ok(HttpResponse::Created().json(response_data))
}

#[get("/api/v1/github-profile")]
pub async fn get_github_profile(
    req: HttpRequest,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let github_profile_service = GithubProfileService::new(&state)?;
    let github_profile = github_profile_service
        .get_by_user_id(user.id.expect("already checked"))
        .await?;

    let response_data = GithubProfileResponse {
        id: github_profile.id,
        user_id: github_profile.user_id.to_string(),
        github_user_id: github_profile.github_user_id.to_string(),
        github_login: github_profile.github_login
    };

    Ok(HttpResponse::Ok().json(response_data))
}


#[delete("/api/v1/github-profile")]
pub async fn delete_github_profile(
    req: HttpRequest,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let github_profile_service = GithubProfileService::new(&state)?;
    github_profile_service.delete(user.id.expect("already checked")).await?;

    Ok(HttpResponse::NoContent().finish())
}
