use std::path::PathBuf;

use anyhow::Context;
use crate::models::user::User;

use sqlx::PgPool;
use uuid::Uuid;


#[tracing::instrument(
    name = "Query all users",
    skip_all
)]
pub async fn get_users_sqlx(
    pool: &PgPool
) -> Result<Vec<User>, anyhow::Error> {
    match sqlx::query_as!(
        User, 
        r#"SELECT 
        user_id,
        first_name,
        last_name,
        email,
        password_hash,
        employee_number,
        active,
        verified,
        picture,
        department,
        role,
        created_at,
        updated_at
        FROM users"#
        )
        .fetch_all(pool)
        .await
    {
        Ok(users) => Ok(users),
        Err(e) => Err(e.into()),
    }
}

#[tracing::instrument(
    name = "Query user",
    skip(pool)
)]
pub async fn get_user_by_id_sqlx(
    pool: &PgPool,
    user_id: &Uuid,
) -> Result<Option<User>, anyhow::Error>
{
    let user: Option<User> = sqlx::query_as!(
        User,
        r#"SELECT 
        user_id,
        first_name,
        last_name,
        email,
        password_hash,
        employee_number,
        active,
        verified,
        picture,
        department,
        role,
        created_at,
        updated_at
        FROM users WHERE user_id = $1"#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to get query")?;

    Ok(user)
}

#[tracing::instrument(
    name = "Query user role",
    skip(pool)
)]
pub async fn get_user_role_sqlx(
    pool: &PgPool,
    user_id: &Uuid,
) -> Result<Option<String>, sqlx::Error> {
    struct Role{ pub role: String }
    let role = sqlx::query_as!(
        Role,
        r#"
        SELECT 
        role
        FROM users
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;


    let role = 
    match role {
        Some(role) => Some(role.role),
        None => None,
    };

    Ok(role)
}


#[tracing::instrument(
    name = "Delete user query",
    skip(pool)
)]
pub async fn delete_user_by_id_sqlx(
    pool: &PgPool,
    user_id: &Uuid,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM users
        WHERE user_id = $1
        "#,
        user_id)
        .execute(pool)
        .await
        .context("Failed to get query")?;

    
    if query.rows_affected() == 0 {
        return Err(anyhow::anyhow!("Non existing user"));
    }

    Ok(())
}
