use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GITrelloError {
    #[error("unexpected error occurred when making Http Request")]
    HttpRequestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("unexpected error occurred when using GitHubAPIClient")]
    GitHubAPIClientError {
        message: String,
    }
}
