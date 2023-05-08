use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;

use crate::api_response::{ApiResponse, e500, e404};
  
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Test {
    pub frase: String,
    pub año: i16,
}



#[tracing::instrument(
    name = "Get tests",
    skip(pool)
)]
pub async fn get_tests(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Get user making the request

    let test = get_test_sqlx(&pool).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Vec<Test>>::new()
        .with_message("Test")
        .with_data(test)
        .to_resp();
        
        return Ok(api_response);
}

#[tracing::instrument(
    name = "Query struct test",
    skip(pool)
)]
pub async fn get_test_sqlx(
    pool: &PgPool,
) -> Result<Vec<Test>, anyhow::Error> {
    let tests: Vec<Test> = sqlx::query_as!(
        Test,
        r#"
        SELECT *
        FROM test
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to execute query")?;

    Ok(tests)
}

#[tracing::instrument(
    name = "insert tests",
    skip(pool)
)]
pub async fn post_test(
    pool: web::Data<PgPool>,
    body: web::Json<Test>,
) -> Result<HttpResponse, actix_web::Error> {
    // Get user making the request
    //
    let body = body.into_inner();

    let test = insert_test_sqlx(&pool, body).await
        .map_err(|_| e500())?;

    let api_response = ApiResponse::<Test>::new()
        .with_message("Nuevo Test")
        .with_data(test)
        .to_resp();
        
        return Ok(api_response);
}


#[tracing::instrument(
    name = "Query struct test",
    skip(pool)
)]
pub async fn insert_test_sqlx(
    pool: &PgPool,
    new_test: Test,
) -> Result<Test, anyhow::Error> {
    let test: Test = sqlx::query_as!(
        Test,
        r#"
        INSERT INTO test
        (frase, año)
        VALUES($1, $2)
        RETURNING *
        "#,
        new_test.frase,
        new_test.año,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(test)
}
