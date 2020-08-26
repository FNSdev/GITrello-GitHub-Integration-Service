use actix_threadpool::BlockingError;
use actix_web::{web, HttpResponse, HttpRequest};

use crate::entities::user::User;
use crate::models::github_profile::NewGithubProfile;
use crate::services::github_api_client::GitHubAPIClient;
use crate::services::repositories::github_profile::GithubProfileRepository;
use crate::state::State;
use crate::value_objects::request_data::github_profile::NewGithubProfileRequest;
use crate::errors::GITrelloError;

#[post("/api/v1/github-profiles")]
pub async fn create_github_profile(
    req: HttpRequest,
    json: web::Json<NewGithubProfileRequest>,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError> {
    let user = User::from_request_extensions(req.extensions());
    if !user.is_authenticated() {
        return Err(GITrelloError::NotAuthenticated)
    }

    let connection = state.db_pool
        .get()
        .map_err(|source| GITrelloError::R2D2Error { source })?;

    let github_api_client = GitHubAPIClient::new(json.access_token.as_str());
    let github_user = github_api_client.get_user().await;

    return if let Ok(github_user) = github_user {
        let data = NewGithubProfile {
            user_id: user.id.expect("is_authenticated() must be checked earlier"),
            github_user_id: github_user.id,
            github_login: github_user.login,
            access_token: String::from(&json.access_token),
        };

        let github_profile =
            web::block(move || {
                let repository = GithubProfileRepository::new(&connection);
                repository.create(&data)
            })
            .await;

        match github_profile {
            Ok(github_profile) => {
                Ok(HttpResponse::Created().json(github_profile))
            }
            Err(e) => {
                match e {
                    BlockingError::Error(e) => {
                        Err(e)
                    }
                    BlockingError::Canceled => {
                        Err(GITrelloError::InternalError)
                    }
                }
            }
        }
    } else {
        Err(GITrelloError::InternalError)
    }
}
