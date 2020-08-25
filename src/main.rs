#[macro_use] extern crate actix_web;
#[macro_use] extern crate diesel;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::result::Result;

use actix_web::{App, HttpResponse, HttpServer, HttpRequest, Responder};

use state::get_state;

mod errors;
mod middlewares;
mod models;
mod schema;
mod services;
mod state;
mod value_objects;

#[get("/test")]
async fn test(req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let i64_extension = match extensions.get::<i64>() {
        Some(val) => *val,
        _ => -1
    };
    println!("{}", i64_extension);

    let service = services::github_api_client::GitHubAPIClient::new(
        "42c65d6caafeb8c9f9e64aa588b29fda6b432d9e"
    );

    let github_user = service.get_user().await;
    match github_user {
        Result::Ok(github_user) => {
            dbg!(github_user);
            HttpResponse::Ok()
        }
        Result::Err(e) => {
            match e {
                errors::GITrelloError::GitHubAPIClientError { message } => {
                    println!("{}", message);
                },
                errors::GITrelloError::HttpRequestError { source } => {
                    dbg!(source);
                },
            }
            HttpResponse::InternalServerError()
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = get_state();

    pretty_env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middlewares::authentication::JWTAuthentication)
            // TODO remove /test endpoint
            .service(test)
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
