use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::errors::ToGITrelloError;
use crate::models::github_profile::NewGithubProfile;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::GithubProfileRepository;
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

    let connection = state.get_db_connection()?;

    let github_api_client = GitHubAPIClient::new(json.access_token.as_str());
    let github_user_result = github_api_client.get_user().await;

    if github_user_result.is_err() {
        return Err(GITrelloError::InternalError);
    }

    let github_user = github_user_result.expect("already checked");
    let data = NewGithubProfile {
        user_id: user.id.expect("is_authenticated() must be checked earlier"),
        github_user_id: github_user.id,
        github_login: github_user.login,
        access_token: String::from(&json.access_token),
    };

    let github_profile = web::block(
            move || {
                let repository = GithubProfileRepository::new(&connection);
                repository.create(&data)
            }
        )
        .await
        .map_err(|e| e.move_to_gitrello_error())?;

    Ok(HttpResponse::Created().json(github_profile))
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

    let connection = state.get_db_connection()?;

    let github_profile = web::block(
            move || {
                let repository = GithubProfileRepository::new(&connection);
                repository.get_by_user_id(
                    user.id.expect("is_authenticated() must be checked earlier"),
                )
            }
        )
        .await
        .map_err(|e| e.move_to_gitrello_error())?;

    Ok(HttpResponse::Ok().json(GithubProfileResponse {
        id: github_profile.id,
        user_id: github_profile.user_id.to_string(),
        github_user_id: github_profile.github_user_id.to_string(),
        github_login: github_profile.github_login
    }))
}
