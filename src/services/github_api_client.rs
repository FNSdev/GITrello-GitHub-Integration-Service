use reqwest::header::{HeaderMap, HeaderValue,  AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Error, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json;

use crate::errors::GITrelloError;
use crate::value_objects::github_api::{
    APIError, CreateWebhook, CreateWebhookConfig, GithubUser, Repository, Webhook,
};

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
    ) -> Result<Option<T>, GITrelloError>
        where T: DeserializeOwned
    {
        match response {
            Ok(response) => {
                if response.status() == expected_status_code {
                    if expected_status_code == StatusCode::NO_CONTENT {
                        return Ok(None);
                    }

                    let json = response.json::<T>().await;

                    // Response with valid code
                    json
                        .map(|json| Some(json))
                        .map_err(|source| GITrelloError::GitHubAPIClientError {
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

        let result = self.process_response::<GithubUser>(response, StatusCode::OK).await;
        match result {
            Ok(result) => Ok(result.expect("can not be None")),
            Err(e) => Err(e),
        }
    }

    pub async fn get_repositories(&self) -> Result<Vec<Repository>, GITrelloError> {
        let url = format!("{}/user/repos", GITHUB_API_URL);

        let response = Client::new()
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await;

        let result = self.process_response::<Vec<Repository>>(response, StatusCode::OK).await;
        match result {
            Ok(result) => Ok(result.expect("can not be None")),
            Err(e) => Err(e),
        }
    }

    pub async fn create_webhook(
        &self,
        repository_name: &str,
        repository_owner: &str,
        webhook_url: &str,
    ) -> Result<Webhook, GITrelloError>
    {
        let url = format!("{}/repos/{}/{}/hooks", GITHUB_API_URL, repository_owner, repository_name);
        let body = CreateWebhook {
            config: CreateWebhookConfig {
                url: webhook_url.to_string(),
                content_type: String::from("json"),
            },
            events: vec![String::from("issues"), ],
        };

        let response = Client::new()
            .post(&url)
            .json::<CreateWebhook>(&body)
            .headers(self.headers.clone())
            .send()
            .await;

        let result = self.process_response::<Webhook>(response, StatusCode::CREATED).await;
        match result {
            Ok(result) => Ok(result.expect("can not be None")),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_webhook(
        &self,
        repository_name: &str,
        repository_owner: &str,
        webhook_id: i64,
    ) -> Result<bool, GITrelloError>
    {
        let url = format!("{}/repos/{}/{}/hooks/{}", GITHUB_API_URL, repository_owner, repository_name, webhook_id);

        let response = Client::new()
            .delete(&url)
            .headers(self.headers.clone())
            .send()
            .await;

        let result = self.process_response::<bool>(response,StatusCode::NO_CONTENT).await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }
}
