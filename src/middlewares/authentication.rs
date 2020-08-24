use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use actix_web::web::Data;

use crate::state::State;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i64,
}

fn get_claims(token: &str, secret: &[u8]) -> Option<Claims> {
    let token = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation {validate_exp: false, ..Default::default()},
    );

    match token {
        Ok(token) => Some(token.claims),
        Err(e) => {
            error!("{}", e);
            None
        },
    }
}

pub struct JWTAuthentication;

impl<S, B> Transform<S> for JWTAuthentication
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JWTAuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JWTAuthenticationMiddleware { service })
    }
}

pub struct JWTAuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service for JWTAuthenticationMiddleware<S>
    where
        S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    // TODO parse header value in a separate function & add tests
    // TODO insert custom User type instead of i64
    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();
        let value = headers.get("Authorization");
        match value {
            Some(auth) => {
                let header_value = auth.to_str().unwrap();
                let parts: Vec<&str> = header_value.split(" ").collect();

                if parts.len() != 2 || parts[0] != "Bearer"{
                    warn!("Invalid Authorization header {}", header_value);
                }
                else {
                    let state: Option<Data<State>> = req.app_data();
                    match state {
                        Some(state) => {
                            let claims = get_claims(parts[1], state.secret.as_ref());
                            match claims {
                                Some(claims) => {
                                    let mut extensions = req.extensions_mut();
                                    extensions.insert::<i64>(claims.user_id);
                                },
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

#[test]
fn test_get_claims() {
    use jsonwebtoken::{encode, Header, EncodingKey};

    let claims = Claims {user_id: 42};
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();

    let decoded_claims = get_claims(token.as_str(), "secret".as_ref());

    assert!(decoded_claims.is_some());
    assert_eq!(decoded_claims.unwrap().user_id, 42);
}

#[test]
fn test_get_claims_invalid_token() {
    use jsonwebtoken::{encode, Header, EncodingKey};

    let claims = Claims {user_id: 42};
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();

    assert!(get_claims(token.as_str(), "wrong_secret".as_ref()).is_none());
}
