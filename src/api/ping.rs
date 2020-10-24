use actix_web::{HttpRequest, Responder};

use crate::entities::user::User;

#[get("/ping")]
pub async fn ping(req: HttpRequest) -> impl Responder {
    let user = User::from_request_extensions(req.extensions());
    match user.is_authenticated() {
        true => format!("Hello, GITrello User {}", user.id.expect("already checked")),
        false => String::from("Hello, Anonymous"),
    }
}
