use actix_web::{web, HttpResponse, HttpRequest};
use futures::StreamExt;

use crate::services::github_webhook_service::GithubWebhookProcessingService;
use crate::state::State;
use crate::errors::GITrelloError;

#[post("/api/v1/webhook")]
pub async fn github_webhook(
    req: HttpRequest,
    mut body: web::Payload,
    state: web::Data<State>,
) -> Result<HttpResponse, GITrelloError>
{
    let headers = req.headers();
    let event = headers.get("X-GitHub-Event");

    return match event {
        None => Ok(HttpResponse::Ok().finish()),
        Some(event_header_value) => {
            let event_name = event_header_value
                .to_str()
                .map_err(|_| GITrelloError::InternalError)?;

            let mut bytes = web::BytesMut::new();
            while let Some(item) = body.next().await {
                bytes.extend_from_slice(&item.map_err(|_| GITrelloError::InternalError)?);
            }

            let service = GithubWebhookProcessingService::new(&state);
            service.process(event_name, bytes).await?;

            Ok(HttpResponse::Ok().finish())
        }
    }
}
