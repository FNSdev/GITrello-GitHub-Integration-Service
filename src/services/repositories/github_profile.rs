use diesel::{insert_into, RunQueryDsl};
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

    pub fn create(&self, data: &NewGithubProfile) -> Result<GithubProfile, GITrelloError>{
        use crate::schema::github_profile::dsl::*;

        insert_into(github_profile)
            .values(data)
            .get_result(self.connection)
            .map_err(|source| GITrelloError::DieselError { source })
    }
}
