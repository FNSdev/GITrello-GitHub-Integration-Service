#[macro_use] extern crate diesel;
#[macro_use] extern crate log;
extern crate pretty_env_logger;

use actix_web::{App, HttpResponse, HttpServer, web, HttpRequest};

use state::get_state;

mod middlewares;
mod models;
mod schema;
mod state;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let state = get_state();

    pretty_env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middlewares::authentication::JWTAuthentication)
            // TODO remove /test endpoint
            .service(
                web::resource("/test")
                    .route(web::post().to(|req: HttpRequest| {
                        let extensions = req.extensions();
                        let i64_extension = match extensions.get::<i64>() {
                            Some(val) => *val,
                            _ => -1
                        };
                        println!("{}", i64_extension);
                        HttpResponse::Ok()
                    }))
            )
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
