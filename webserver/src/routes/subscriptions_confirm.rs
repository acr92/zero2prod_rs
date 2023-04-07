use actix_web::{web, HttpResponse};
use actix_web_codegen::get;
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::SubscribeError;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm new subscriber", skip(parameters, pool))]
#[get("/subscriptions/confirm")]
pub async fn subscriptions_confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscribeError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to fetch subscriber id from token")?
        .context("Unable to find subscription token")?;

    confirm_subscriber(&pool, id)
        .await
        .context("Failed to confirm subscriber")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Confirm subscriber in database", skip(pool))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET confirmed = true
        WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Fetch subscriber id from token", skip(pool, token))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT id FROM subscription_tokens
        WHERE token = $1
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.id))
}
