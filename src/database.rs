use dotenv::dotenv;
use std::env;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

pub type DbPool = PgPool;
/// Establishes a connection to the database
///
/// # Arguments
/// None
///
/// # Returns
/// (PgPool): Database pool
pub async fn establish_connection() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    debug!("Connecting to Database url: {}", database_url);

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    pool
}
