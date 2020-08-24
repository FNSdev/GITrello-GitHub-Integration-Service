#[macro_use] extern crate diesel;
extern crate pretty_env_logger;

use actix_web::{App, HttpServer};

use state::get_state;

mod state;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = get_state();

    pretty_env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
