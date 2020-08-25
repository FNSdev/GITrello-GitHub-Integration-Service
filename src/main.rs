#[macro_use] extern crate actix_web;
#[macro_use] extern crate diesel;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use actix_web::{App, HttpServer};

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
            .service(api::github_profile::create_github_profile)
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
