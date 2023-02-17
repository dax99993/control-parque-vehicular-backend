use crate::api_response::{e401, e500, ApiResponse, e409};
use crate::models::user::SignupUser;
use crate::email_client::EmailClient;
use crate::authentication::password::compute_password_hash;
use crate::telemetry::spawn_blocking_with_tracing;
use crate::startup::ApplicationBaseUrl;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use sqlx::{PgPool, Transaction, Postgres};
use secrecy::ExposeSecret;
use uuid::Uuid;
use validator::Validate;


#[tracing::instrument(
    name = "Signup user",
    skip(body, pool, email_client, base_url)
)]
pub async fn signup_user(
    pool: web::Data<PgPool>,
    body: web::Json<SignupUser>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, actix_web::Error> {

    /* Validate body signup */
    let signup_user = body.into_inner();
    signup_user.validate().map_err(|_| e401().with_message("Invalid body"))?;

    /* check passwords match */
    if signup_user.password.expose_secret() !=
       signup_user.re_password.expose_secret() {
        return Err(e401().with_message("Passwords dont match"))?;
    }

    if signup_user.password.expose_secret().len() < 6 ||
       signup_user.password.expose_secret().len() > 255 {
        return Err(e401().with_message("Password should be between 6 and 255 characters"))?;
    }

    let mut transaction = pool.begin()
        .await
        .map_err(|_| e500())?;

    /* verify if user with given email exists, in case it does conflict */
    let user_exists = exists_user_with_email(&mut transaction, &signup_user.email)
        .await
        .context("Failed to query existing user.")
        .map_err(|_| e500())?;

    if user_exists {
        return Err(e409().with_message("User already exits"))?
    }

    // insert new user in database
    let user_email = signup_user.email.clone();
    let user_id = insert_user(&mut transaction, signup_user)
        .await
        .map_err(|_| e500())?;
    tracing::Span::current()
        .record("user_id", &tracing::field::display(&user_id));
    // generate signup token
    let signup_token = generate_signup_token();
    store_token(&mut transaction, user_id, &signup_token)
        .await
        .map_err(|_| e500())?;
    transaction.commit()
        .await
        .map_err(|_| e500())?;
    //send email to verify user
    send_confirmation_email(&email_client, &base_url.0, &user_email, &signup_token)
        .await
        .map_err(|_| e500())?;

    // Maybe change Created to Accepted due to email not being sent
    Ok(
        ApiResponse::<()>::new().with_message("User created").to_resp()
     )
}


#[tracing::instrument(
    name = "Querying user existence",
    skip(email, transaction)
)]
async fn exists_user_with_email(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT user_id FROM users
            WHERE email = $1
        )
        "#,
        email
        )
        .fetch_one(transaction)
        .await?;

    let user_exists = row.exists.unwrap();

    Ok(user_exists)
}

#[tracing::instrument(
    name = "insert new user",
    skip(user, transaction)
)]
async fn insert_user(
    transaction: &mut Transaction<'_, Postgres>,
    user: SignupUser,
) -> Result<uuid::Uuid, anyhow::Error> {
    let password_hash = spawn_blocking_with_tracing(
        move || compute_password_hash(user.password)
        )
        .await?
        .context("Failed to hash password")?;

    let uuid = Uuid::new_v4();
    let row = sqlx::query!(
        r#"
        INSERT INTO users
        (user_id, first_name, last_name, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING user_id
        "#,
        uuid,
        user.first_name,
        user.last_name,
        user.email,
        password_hash.expose_secret(),
    )
    .fetch_one(transaction)
    .await
    .context("Failed to performed a query to retrieve stored new user")?;
    //.map(|row| row.user_id);
    
    Ok(row.user_id)
}

fn generate_signup_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store signup token in database",
    skip(signup_token, transaction)
)]
async fn store_token (
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    signup_token: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO signup_tokens
        (signup_token, user_id)
        VALUES ($1, $2)
        "#,
        signup_token,
        user_id,
    )
    .execute(transaction)
    .await
    .context("Failed to performed a query to retrieve stored new user")?;
    //.map(|row| row.user_id);
    
    Ok(())
}


#[tracing::instrument(
    name = "Send confirmation email to new user",
    skip(email_client, user_email)
)]
async fn send_confirmation_email (
    email_client: &EmailClient,
    base_url: &str,
    user_email: &str,
    signup_token: &str,
) -> Result<(), anyhow::Error> {
    let confirmation_link = format!(
        "{}/api/auth/signups/confirm?signup_token={}",
        base_url,
        signup_token,
        );

    email_client.send_email(
        user_email,
        "Bienvenido",
        &format!("Bienvenido a Control Parque Vehicular!<br />\
                 Haz click <a href=\"{}\">aqui</a> para confirmar cuenta.",
                 confirmation_link),
        &format!("Bienvenido a Control Parque Vehicular!\nVisita {} para confirmar cuenta.",
                confirmation_link),
    )
    .await
}
