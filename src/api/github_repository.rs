use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::errors::{GITrelloError, ToGITrelloError};
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::GithubProfileRepository;
use crate::state::State;
use crate::value_objects::response_data::github_repository::GetGithubRepositoryResponse;

#[get("/api/v1/github-repositories")]
pub async fn get_github_repositories(
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

    let github_service = GitHubAPIClient::new(github_profile.access_token.as_str());
    let repositories = github_service.get_repositories().await?;

    let mut response_data: Vec<GetGithubRepositoryResponse> = Vec::new();
    for repo in repositories {
        response_data.push(GetGithubRepositoryResponse {
            id: repo.id.to_string(),
            name: repo.name,
        });
    }

    Ok(HttpResponse::Ok().json(response_data))
}
