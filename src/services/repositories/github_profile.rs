use diesel::{
    insert_into, result::DatabaseErrorKind, result::Error, RunQueryDsl, QueryDsl, ExpressionMethods,
};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

use crate::errors::GITrelloError;
use crate::models::github_profile::{GithubProfile, NewGithubProfile};

pub struct GithubProfileRepository<'a> {
    connection: &'a PooledConnection<ConnectionManager<PgConnection>>,
}

impl <'a> GithubProfileRepository<'a> {
    pub fn new(connection: &'a PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { connection }
    }

    pub fn create(&self, data: &NewGithubProfile) -> Result<GithubProfile, GITrelloError> {
        use crate::schema::github_profile::dsl::*;

        insert_into(github_profile)
            .values(data)
            .get_result(self.connection)
            .map_err(|source| {
                match source {
                    Error::DatabaseError (DatabaseErrorKind::UniqueViolation, error_info) => {
                        GITrelloError::AlreadyExists { message: String::from(error_info.message()) }
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }

    pub fn get_by_user_id(&self, user_id: i64) -> Result<GithubProfile, GITrelloError> {
        use crate::schema::github_profile::{table, user_id as user_id_column};

        table
            .filter(user_id_column.eq(user_id))
            .first::<GithubProfile>(self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("github_profile for user {} does not exist", user_id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }
}
