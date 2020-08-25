use std::cell::Ref;

use actix_web::dev::Extensions;

pub struct User {
    pub id: Option<i64>,
}

impl User {
    pub fn from_request_extensions(extensions: Ref<Extensions>) -> Self {
        let user_id: Option<i64> = match extensions.get::<i64>() {
            Some(val) => Some(*val),
            _ => None,
        };

        Self { id: user_id }
    }

    pub fn is_authenticated(&self) -> bool {
        self.id.is_some()
    }
}

#[test]
fn test_is_authenticated() {
    let user = User { id: Some(42) };
    assert_eq!(user.is_authenticated(), true);
}

#[test]
fn test_is_not_authenticated() {
    let user = User { id: None };
    assert_eq!(user.is_authenticated(), false);
}
