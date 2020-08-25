use std::fmt;

use actix_http::{ResponseBuilder, http};
use actix_web::{error, HttpResponse};
use diesel;
use r2d2;
use reqwest;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    pub error_message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum GITrelloError {
    DieselError {
        #[from]
        source: diesel::result::Error,
    },
    R2D2Error {
        #[from]
        source: r2d2::Error,
    },
    HttpRequestError {
        #[from]
        source: reqwest::Error,
    },
    GitHubAPIClientError {
        message: String,
    },
    NotAuthenticated,
    InternalError,
}

impl fmt::Display for GITrelloError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DieselError { source} => {
                write!(f, "{}", source.to_string())
            },
            Self::R2D2Error { source} => {
                write!(f, "{}", source.to_string())
            },
            Self::HttpRequestError { source } => {
                write!(f, "{}", source.to_string())
            },
            Self::GitHubAPIClientError { message } => {
                write!(f, "{}", message)
            },
            Self::NotAuthenticated => write!(f, "Authentication required"),
            Self::InternalError => write!(f, "Internal Server Error"),
        }
    }
}

impl error::ResponseError for GITrelloError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            Self::NotAuthenticated => http::StatusCode::UNAUTHORIZED,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        error!("{}", self.to_string());
        ResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(ErrorResponse { error_message: self.to_string() })
    }
}
