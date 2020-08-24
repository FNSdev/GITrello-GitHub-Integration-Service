use std::env;

use dotenv::dotenv;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone, PartialEq)]
pub enum Environment {
    Dev,
    Test,
    Prod,
}

#[derive(Clone)]
pub struct State {
    pub environment: Environment,
    pub db_pool: DbPool,
}

pub fn get_state() -> State {
    let environment = match env::var("ENVIRONMENT") {
        Ok(value) => {
            match value.as_str() {
                "DEV" => Environment::Dev,
                "TEST" => Environment::Test,
                "PROD" => Environment::Prod,
                _ => panic!("{} is not a valid ENVIRONMENT option", value)
            }
        },
        Err(_) => Environment::Dev
    };

    if environment == Environment::Dev {
        dotenv().ok();
    }

    let db_url = env::var("DB_URL").expect("DB_URL");
    let db_pool_min_idle: u32 = env::var("DB_POOL_MIN_IDLE")
        .expect("DB_POOL_MIN_IDLE")
        .parse()
        .expect("DB_POOL_MIN_IDLE is not a valid integer");
    let db_pool_max_size: u32 = env::var("DB_POOL_MAX_SIZE")
        .expect("DB_POOL_MAX_SIZE")
        .parse()
        .expect("DB_POOL_MAX_SIZE is not a valid integer");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let db_pool = r2d2::Pool::builder()
        .min_idle(Some(db_pool_min_idle))
        .max_size(db_pool_max_size)
        .build(manager)
        .expect("Failed to create database connection pool.");

    let state = State {environment, db_pool};
    state
}
