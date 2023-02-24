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
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM users
        WHERE user_id = $1
        "#,
        user_id)
        .execute(pool)
        .await
        .context("Failed to get query")?;

    Ok(query.rows_affected() != 0)
}

#[tracing::instrument(
    name = "update user query",
    skip_all
)]
pub async fn update_user_sqlx(
    pool: &PgPool,
    user: User,
) -> Result<User, anyhow::Error> {
    let user = sqlx::query_as!(
        User, 
        r#"
        UPDATE users
        SET
        first_name = $2,
        last_name = $3,
        employee_number = $4,
        active = $5,
        verified = $6,
        department = $7,
        role = $8,
        email = $9,
        updated_at = now()
        WHERE user_id = $1
        RETURNING *
        "#,
        user.user_id,
        user.first_name,
        user.last_name,
        user.employee_number,
        user.active,
        user.verified,
        user.department,
        user.role,
        user.email,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(user)
}

#[tracing::instrument(
    name = "update user picture query",
    skip_all
)]
pub async fn update_user_picture_sqlx(
    pool: &PgPool,
    user: User,
) -> Result<User, anyhow::Error> {
    let user = sqlx::query_as!(
        User, 
        r#"
        UPDATE users
        SET
        picture = $2,
        updated_at = now()
        WHERE user_id = $1
        RETURNING *
        "#,
        user.user_id,
        user.picture,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(user)
}
