use actix::{Actor, Context, Handler, Message};
use diesel::{
    delete, insert_into, result::DatabaseErrorKind, result::Error, RunQueryDsl, QueryDsl,
    ExpressionMethods,
};

use crate::errors::GITrelloError;
use crate::models::github_profile::{GithubProfile, NewGithubProfile};
use crate::state::DbConnection;

pub struct GithubProfileRepository {
    connection: DbConnection,
}

impl GithubProfileRepository {
    pub fn new(connection: DbConnection) -> Self {
        Self { connection }
    }

    pub fn create(&self, data: &NewGithubProfile) -> Result<GithubProfile, GITrelloError> {
        use crate::schema::github_profile::dsl::*;

        insert_into(github_profile)
            .values(data)
            .get_result(&self.connection)
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
            .first::<GithubProfile>(&self.connection)
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

    pub fn get_by_github_user_id(&self, github_user_id: i64) -> Result<GithubProfile, GITrelloError> {
        use crate::schema::github_profile::{table, github_user_id as github_user_id_column};

        table
            .filter(github_user_id_column.eq(github_user_id))
            .first::<GithubProfile>(&self.connection)
            .map_err(|source| {
                match source {
                    Error::NotFound => GITrelloError::NotFound {
                        message: String::from(
                            format!("github_profile for GitHub user {} does not exist", github_user_id),
                        )
                    },
                    _ => GITrelloError::DieselError { source }
                }
            })
    }

    pub fn delete(&self, id: i32) -> Result<(), GITrelloError> {
        use crate::schema::github_profile::{table, id as id_column};

        delete(table.filter(id_column.eq(id))).execute(&self.connection)?;
        Ok(())
    }
}

impl Actor for GithubProfileRepository {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Result<GithubProfile, GITrelloError>")]
pub struct CreateGithubProfileMessage {
    pub data: NewGithubProfile,
}

impl Handler<CreateGithubProfileMessage> for GithubProfileRepository {
    type Result = Result<GithubProfile, GITrelloError>;

    fn handle(
        &mut self,
        msg: CreateGithubProfileMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.create(&msg.data)
    }
}

#[derive(Message)]
#[rtype(result = "Result<GithubProfile, GITrelloError>")]
pub struct GetGithubProfileByUserIdMessage {
    pub user_id: i64,
}

impl Handler<GetGithubProfileByUserIdMessage> for GithubProfileRepository {
    type Result = Result<GithubProfile, GITrelloError>;

    fn handle(
        &mut self,
        msg: GetGithubProfileByUserIdMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.get_by_user_id(msg.user_id)
    }
}

#[derive(Message)]
#[rtype(result = "Result<GithubProfile, GITrelloError>")]
pub struct GetGithubProfileByGithubUserIdMessage {
    pub github_user_id: i64,
}

impl Handler<GetGithubProfileByGithubUserIdMessage> for GithubProfileRepository {
    type Result = Result<GithubProfile, GITrelloError>;

    fn handle(
        &mut self,
        msg: GetGithubProfileByGithubUserIdMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.get_by_github_user_id(msg.github_user_id)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), GITrelloError>")]
pub struct DeleteGithubProfileMessage {
    pub id: i32,
}

impl Handler<DeleteGithubProfileMessage> for GithubProfileRepository {
    type Result = Result<(), GITrelloError>;

    fn handle(
        &mut self,
        msg: DeleteGithubProfileMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result
    {
        self.delete(msg.id)
    }
}
