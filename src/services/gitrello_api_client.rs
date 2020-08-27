use reqwest::header::{HeaderMap, HeaderValue,  AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Error, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json;

use crate::errors::GITrelloError;
use crate::value_objects::gitrello_api::{APIError, BoardPermissions, GetBoardPermissionsRequest};

pub struct GITRelloAPIClient<'a> {
    gitrello_url: &'a str,
    headers: HeaderMap,
}

impl<'a> GITRelloAPIClient<'a> {
    pub fn new(gitrello_url: &'a str) -> Self {
        let user_agent_header_value = HeaderValue::from_str(
            "GITRello GitHub Integration Service",
            )
            .expect("User Agent header should be valid");

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, user_agent_header_value);

        Self { headers, gitrello_url }
    }

    pub fn with_access_token(gitrello_url: &'a str, access_token: &str) -> Self {
        let authorization_header_value = HeaderValue::from_str(
                format!("Token {}", access_token).as_str(),
            )
            .expect("Authorization header should be valid");

        let mut service = Self::new(gitrello_url);
        service.headers.insert(AUTHORIZATION, authorization_header_value);
        service
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
                                    Err(GITrelloError::GITrelloAPIClientError {
                                        message: api_error.error_message,
                                    })
                                },
                                // Response with invalid code can not be deserialized into APIError
                                _ => {
                                    Err(GITrelloError::GITrelloAPIClientError {
                                        message: response_string,
                                    })
                                }
                            }
                        },
                        // Response with invalid code can not be read into String
                        Err(e) => {
                            Err(GITrelloError::GITrelloAPIClientError { message: e.to_string() })
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

    pub async fn get_board_permissions(
        &self,
        user_id: i64,
        board_id: i64,
    ) -> Result<BoardPermissions, GITrelloError>
    {
        let url = format!("{}/{}", self.gitrello_url, "user");

        let response = Client::new()
            .get(&url)
            .json(&GetBoardPermissionsRequest {board_id, user_id})
            .headers(self.headers.clone())
            .send()
            .await;

        self.process_response::<BoardPermissions>(response, StatusCode::OK).await
    }
}
