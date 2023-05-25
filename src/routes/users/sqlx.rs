use anyhow::Context;
//use crate::models::user::User;
use common::models::user::{Usuario, UsuarioRol};

use sqlx::PgPool;
use uuid::Uuid;


#[tracing::instrument(
    name = "Query todos los usuarios",
    skip_all
)]
pub async fn obtener_usuarios_sqlx(
    pool: &PgPool
) -> Result<Vec<Usuario>, anyhow::Error> {
    match sqlx::query_as!(
        Usuario, 
        r#"SELECT 
        usuario_id,
        nombres,
        apellidos,
        email,
        password_hash,
        numero_empleado,
        activo,
        verificado,
        imagen,
        COALESCE(departamentos.nombre, 'Sin asignar') as "departamento!",
        rol as "rol!: UsuarioRol",
        creado_en,
        modificado_en
        FROM usuarios LEFT JOIN departamentos
        ON usuarios.departamento = departamentos.id"#
        )
        .fetch_all(pool)
        .await
    {
        Ok(users) => Ok(users),
        Err(e) => Err(e.into()),
    }
}

#[tracing::instrument(
    name = "Query usuario por id",
    skip(pool)
)]
pub async fn obtener_usuario_por_id_sqlx(
    pool: &PgPool,
    usuario_id: &Uuid,
) -> Result<Option<Usuario>, anyhow::Error>
{
    let usuario: Option<Usuario> = sqlx::query_as!(
        Usuario,
        r#"SELECT 
        usuario_id,
        nombres,
        apellidos,
        email,
        password_hash,
        numero_empleado,
        activo,
        verificado,
        imagen,
        COALESCE(departamentos.nombre, 'Sin asignar') as "departamento!",
        rol as "rol!: UsuarioRol",
        creado_en,
        modificado_en
        FROM usuarios LEFT JOIN departamentos
        ON usuarios.departamento = departamentos.id
        WHERE usuario_id = $1"#,
        usuario_id,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to get query")?;

    Ok(usuario)
}


#[tracing::instrument(
    name = "Query rol del usuario",
    skip(pool)
)]
pub async fn obtener_usuario_rol_sqlx(
    pool: &PgPool,
    usuario_id: &Uuid,
) -> Result<Option<UsuarioRol>, sqlx::Error> {
    struct Rol{ pub rol: UsuarioRol }
    let rol = sqlx::query_as!(
        Rol,
        r#"
        SELECT 
        rol as "rol!: UsuarioRol"
        FROM usuarios
        WHERE usuario_id = $1
        "#,
        usuario_id,
    )
    .fetch_optional(pool)
    .await?;

    let rol = 
    match rol {
        Some(rol) => Some(rol.rol),
        None => None,
    };

    Ok(rol)
}


#[tracing::instrument(
    name = "Query SQLX Borrar usuario",
    skip(pool)
)]
pub async fn borrar_usuario_por_id_sqlx(
    pool: &PgPool,
    usuario_id: &Uuid,
) -> Result<bool, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM usuarios
        WHERE usuario_id = $1
        "#,
        usuario_id)
        .execute(pool)
        .await
        .context("Failed to get query")?;

    Ok(query.rows_affected() != 0)
}

#[tracing::instrument(
    name = "Query actualizar usuario",
    skip_all
)]
pub async fn actualizar_usuario_sqlx(
    pool: &PgPool,
    usuario: Usuario,
) -> Result<Usuario, anyhow::Error> {
    let usuario = sqlx::query_as!(
        Usuario, 
        r#"
        UPDATE usuarios
        SET
        nombres = $2,
        apellidos = $3,
        numero_empleado = $4,
        activo = $5,
        verificado = $6,
        departamento = d.id,
        rol = $8,
        email = $9,
        modificado_en = now()
        FROM departamentos d
        WHERE usuario_id = $1 AND d.nombre = $7
        RETURNING 
            usuario_id,
            nombres,
            apellidos,
            email,
            password_hash,
            numero_empleado,
            activo,
            verificado,
            imagen,
            d.nombre as "departamento!",
            rol as "rol!: UsuarioRol",
            creado_en,
            modificado_en
        "#,
        usuario.usuario_id,
        usuario.nombres,
        usuario.apellidos,
        usuario.numero_empleado,
        usuario.activo,
        usuario.verificado,
        usuario.departamento,
        usuario.rol as UsuarioRol,
        usuario.email,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(usuario)
}

#[tracing::instrument(
    name = "Query actualizar imagen del usuario",
    skip_all
)]
pub async fn actualizar_imagen_usuario_sqlx(
    pool: &PgPool,
    usuario: Usuario,
) -> Result<Usuario, anyhow::Error> {
    let usuario = sqlx::query_as!(
        Usuario, 
        r#"
        WITH actualizado AS(
        UPDATE usuarios
        SET
        imagen = $2,
        modificado_en = now()
        WHERE usuario_id = $1
        RETURNING *
        )
        SELECT 
        usuario_id,
        nombres,
        apellidos,
        email,
        password_hash,
        numero_empleado,
        activo,
        verificado,
        imagen,
        COALESCE(departamentos.nombre, 'Sin asignar') as "departamento!",
        rol as "rol!: UsuarioRol",
        creado_en,
        modificado_en
        FROM actualizado LEFT JOIN departamentos
        ON actualizado.departamento = departamentos.id
        "#,
        usuario.usuario_id,
        usuario.imagen,
    )
    .fetch_one(pool)
    .await
    .context("Failed to execute query")?;

    Ok(usuario)
}
