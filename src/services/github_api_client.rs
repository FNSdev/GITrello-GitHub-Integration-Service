use reqwest::header::{HeaderMap, HeaderValue,  AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Error, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json;

use crate::errors::GITrelloError;
use crate::value_objects::github_api::{APIError, GithubUser, Repository};

const GITHUB_API_URL: &str = "https://api.github.com";

pub struct GitHubAPIClient {
    headers: HeaderMap,
}

impl GitHubAPIClient {
    pub fn new(access_token: &str) -> Self {
        let authorization_header_value = HeaderValue::
        from_str(
            format!("Token {}", access_token).as_str())
            .expect("Authorization header should be valid");

        let user_agent_header_value = HeaderValue::
        from_str(
            "GITRello GitHub Integration Service")
            .expect("User Agent header should be valid");

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, authorization_header_value);
        headers.insert(USER_AGENT, user_agent_header_value);

        Self { headers }
    }

    async fn process_response<T>(
        &self,
        response: Result<Response, Error>,
        expected_status_code: StatusCode
    ) -> Result<T, GITrelloError>
        where T: DeserializeOwned
    {
        match response {
            Ok(response) => {
                if response.status() == expected_status_code {
                    let github_user = response.json::<T>().await;

                    // Response with valid code
                    github_user.map_err(|source| GITrelloError::GitHubAPIClientError {
                        message: source.to_string(),
                    })
                }
                else {
                    let response_string = response.text().await;
                    match response_string {
                        Ok(response_string) => {
                            let api_error: Result<APIError, _> = serde_json::from_str(
                                response_string.as_str(),
                            );
                            match api_error {
                                // Response with invalid code can be deserialized into APIError
                                Ok(api_error) => {
                                    Err(GITrelloError::GitHubAPIClientError {
                                        message: api_error.message,
                                    })
                                },
                                // Response with invalid code can not be deserialized into APIError
                                _ => {
                                    Err(GITrelloError::GitHubAPIClientError {
                                        message: response_string,
                                    })
                                }
                            }
                        },
                        // Response with invalid code can not be read into String
                        Err(e) => {
                            Err(GITrelloError::GitHubAPIClientError { message: e.to_string() })
                        }
                    }
                }
            }
            // HTTP Request failed
            Err(e) => {
                Err(GITrelloError::HttpRequestError {source: e})
            }
        }
    }

    pub async fn get_user(&self) -> Result<GithubUser, GITrelloError> {
        let url = format!("{}/{}", GITHUB_API_URL, "user");

        let response = Client::new()
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await;

        self.process_response::<GithubUser>(response, StatusCode::OK).await
    }

    pub async fn get_repositories(&self) -> Result<Vec<Repository>, GITrelloError> {
        let url = format!("{}/{}", GITHUB_API_URL, "user/repos");

        let response = Client::new()
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await;

        self.process_response::<Vec<Repository>>(response, StatusCode::OK).await
    }
}
