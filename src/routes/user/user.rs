use anyhow::Context;
use crate::models::user::User;

use sqlx::PgPool;
use uuid::Uuid;


async fn get_users(
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

pub async fn get_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
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

async fn get_user_role(
    pool: &PgPool,
    user_id: Uuid,
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
