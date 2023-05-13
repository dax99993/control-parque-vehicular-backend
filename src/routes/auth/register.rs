use crate::api_response::{e401, e500, ApiResponse, e409};
//use crate::models::user::SignupUser;
use common::models::user::SignupUsuario;
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
    name = "Signup usuario",
    skip_all,
    fields(user_id=tracing::field::Empty)
)]
pub async fn signup_user(
    pool: web::Data<PgPool>,
    body: web::Json<SignupUsuario>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, actix_web::Error> {

    /* Validar signup body*/
    let signup_usuario = body.into_inner();
    // Extraer los Error irregresarlos en un mensaje del api
    signup_usuario.validate().map_err(|_| e401().with_message("Invalid body"))?;

    /* checar passwords match */
    /*
    if signup_user.password.expose_secret() !=
       signup_user.re_password.expose_secret() {
        return Err(e401().with_message("Passwords dont match"))?;
    }

    if signup_user.password.expose_secret().len() < 6 ||
       signup_user.password.expose_secret().len() > 255 {
        return Err(e401().with_message("Password should be between 6 and 255 characters"))?;
    }
    */

    let mut transaction = pool.begin()
        .await
        .map_err(|_| e500())?;

    /* verificar si usuario con email este email existe, en caso que si retornar error */
    let usuario_existe = existe_usuario_con_email(&mut transaction, &signup_usuario.email)
        .await
        .map_err(|_| e500())?;

    if usuario_existe {
        return Err(e409().with_message("Ya existe usuario con ese correo electronico"))?
    }

    // insert new user in database
    let usuario_email = signup_usuario.email.clone();
    let usuario_id = insertar_usuario_sqlx(&mut transaction, signup_usuario)
        .await
        .map_err(|_| e500())?;

    tracing::Span::current()
        .record("usuario_id", &tracing::field::display(&usuario_id));

    // generate signup token
    let signup_token = generate_signup_token();
    store_token(&mut transaction, usuario_id, &signup_token)
        .await
        .map_err(|_| e500())?;
    transaction.commit()
        .await
        .map_err(|_| e500())?;

    //send email to verify user
    send_confirmation_email(&email_client, &base_url.0, &usuario_email, &signup_token)
        .await
        .map_err(|_| e500())?;

    // Maybe change Created to Accepted due to email not being sent
    Ok(
        ApiResponse::<()>::new()
            .with_status_code(201)
            .with_message("Usuario creado")
            .to_resp()
     )
}


#[tracing::instrument(
    name = "Querying existencia de usuario",
    skip(email, transaction)
)]
async fn existe_usuario_con_email(
    transaction: &mut Transaction<'_, Postgres>,
    email: &str
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT usuario_id FROM usuarios
            WHERE email = $1
        )
        "#,
        email
        )
        .fetch_one(transaction)
        .await?;

    Ok(row.exists.unwrap())
}

#[tracing::instrument(
    name = "insert new user",
    skip(usuario, transaction)
)]
async fn insertar_usuario_sqlx(
    transaction: &mut Transaction<'_, Postgres>,
    usuario: SignupUsuario,
) -> Result<uuid::Uuid, anyhow::Error> {

    let password_hash = spawn_blocking_with_tracing(
            //move || compute_password_hash(Secret::new(usuario.password))
            move || compute_password_hash(usuario.password.into())
        )
        .await?
        .context("Failed to hash password")?;

    let uuid = Uuid::new_v4();
    let row = sqlx::query!(
        r#"
        INSERT INTO usuarios
        (usuario_id, nombres, apellidos, email, password_hash)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING usuario_id
        "#,
        uuid,
        usuario.nombres,
        usuario.apellidos,
        usuario.email,
        password_hash.expose_secret(),
    )
    .fetch_one(transaction)
    .await
    .context("Failed to performed a query to retrieve stored new user")?;
    //.map(|row| row.user_id);
    
    Ok(row.usuario_id)
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
        (signup_token, usuario_id)
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
