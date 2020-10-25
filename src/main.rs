#[macro_use] extern crate actix_web;
#[macro_use] extern crate diesel;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};

use state::get_state;

mod api;
mod entities;
mod errors;
mod middlewares;
mod models;
mod schema;
mod services;
mod state;
mod value_objects;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = get_state();

    pretty_env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middlewares::authentication::JWTAuthentication)
            .wrap(Cors::new()
                .allowed_origin(state.gitrello_host.as_str())
                .finish()
            )
            .wrap(Logger::default())
            .service(api::board_repository::create_or_update_board_repository)
            .service(api::board_repository::get_board_repository)
            .service(api::board_repository::delete_board_repository)
            .service(api::github_profile::create_github_profile)
            .service(api::github_profile::get_github_profile)
            .service(api::github_repository::get_github_repositories)
            .service(api::github_webhook::github_webhook)
            .service(api::ping::ping)
    })
    .bind("0.0.0.0:8001")?
    .run()
    .await
}
