use actix::Actor;
use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::errors::GITrelloError;
use crate::models::github_profile::GithubProfile;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::{
    GithubProfileRepository, GetGithubProfileByUserIdMessage,
};
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
    let repository_actor = GithubProfileRepository::new(connection).start();
    let github_profile: GithubProfile = repository_actor
        .send(GetGithubProfileByUserIdMessage { user_id: user.id.expect("already checked") })
        .await
        .map_err(|source| GITrelloError::ActorError { source })??;
    
    let github_service = GitHubAPIClient::new(github_profile.access_token.as_str());
    let repositories = github_service.get_repositories().await?;

    let mut response_data: Vec<GetGithubRepositoryResponse> = Vec::new();
    for repo in repositories {
        let parts: Vec<&str> = repo.full_name.split("/").collect();
        response_data.push(GetGithubRepositoryResponse {
            name: parts[1].to_string(),
            owner: parts[0].to_string()
        });
    }

    Ok(HttpResponse::Ok().json(response_data))
}
