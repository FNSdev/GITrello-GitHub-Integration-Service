use std::fmt;

use actix_http::{ResponseBuilder, http};
use actix_threadpool::BlockingError;
use actix_web::{error, HttpResponse};
use diesel;
use r2d2;
use reqwest;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    pub error_message: String,
    pub error_code: i16,
}

pub trait ToGITrelloError {
    fn move_to_gitrello_error(self) -> GITrelloError;
}

impl ToGITrelloError for BlockingError<GITrelloError> {
    fn move_to_gitrello_error(self) -> GITrelloError {
        match self {
            BlockingError::Error(e) => {
                e
            }
            BlockingError::Canceled => {
                GITrelloError::InternalError
            }
        }
    }
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
    GITrelloAPIClientError {
        message: String,
    },
    NotAuthenticated,
    AlreadyExists {
        message: String,
    },
    NotFound {
        message: String,
    },
    PermissionDenied,
    InternalError,
}

impl fmt::Display for GITrelloError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DieselError { source} => write!(f, "{}", source.to_string()),
            Self::R2D2Error { source} => write!(f, "{}", source.to_string()),
            Self::HttpRequestError { source } => write!(f, "{}", source.to_string()),
            Self::GitHubAPIClientError { message } => write!(f, "{}", message),
            Self::GITrelloAPIClientError { message } => write!(f, "{}", message),
            Self::NotAuthenticated => write!(f, "Authentication required"),
            Self::InternalError => write!(f, "Internal Server Error"),
            Self::AlreadyExists { message } => write!(f, "{}", message),
            Self::NotFound { message } => write!(f, "{}", message),
            Self::PermissionDenied  => write!(f, "Permission denied")
        }
    }
}

impl error::ResponseError for GITrelloError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            Self::NotAuthenticated => http::StatusCode::UNAUTHORIZED,
            Self::AlreadyExists { message: _ } => http::StatusCode::BAD_REQUEST,
            Self::NotFound { message: _ } => http::StatusCode::NOT_FOUND,
            Self::PermissionDenied => http::StatusCode::FORBIDDEN,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        error!("{}", self.to_string());

        let error_code = match self {
            Self::DieselError { source: _ } => 100,
            Self::R2D2Error { source: _ } => 101,
            Self::HttpRequestError { source: _ } => 102,
            Self::GitHubAPIClientError { message: _ } => 103,
            Self::NotAuthenticated => 104,
            Self::InternalError => 105,
            Self::AlreadyExists { message: _ } => 106,
            Self::NotFound { message: _ } => 107,
            Self::GITrelloAPIClientError { message: _ } => 108,
            Self::PermissionDenied => 109,
        };

        ResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "application/json")
            .json(ErrorResponse { error_message: self.to_string(), error_code })
    }
}
