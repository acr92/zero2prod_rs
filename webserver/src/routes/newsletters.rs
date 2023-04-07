use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web_codegen::post;
use anyhow::Context;
use sqlx::PgPool;

use crate::domain::SubscriberEmail;
use crate::email::EmailClient;
use crate::telemetry::error_chain_fmt;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize, Debug)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum NewslettersError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
}

impl std::fmt::Debug for NewslettersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for NewslettersError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Sending newsletters",
    skip(payload, pool, email_client),
    fields(
        title = %payload.title,
    )
)]
#[post("/newsletters")]
pub async fn newsletters(
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    payload: Json<FormData>,
) -> Result<HttpResponse, NewslettersError> {
    let subscribers = fetch_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                send_email(&email_client, &payload, subscriber).await?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber, their stored contact details are invalid")
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Sending newsletter email", skip(email_client, payload))]
async fn send_email(
    email_client: &EmailClient,
    payload: &Json<FormData>,
    subscriber: SubscriberEmail,
) -> Result<(), anyhow::Error> {
    email_client
        .send_email(
            &subscriber,
            &payload.title,
            &payload.content.html,
            &payload.content.text,
        )
        .await
        .with_context(move || format!("Failed to send email to {:?}", subscriber))?;
    Ok(())
}

#[tracing::instrument(name = "Fetching confirmed subscribers", skip(pool))]
async fn fetch_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<SubscriberEmail, anyhow::Error>>, anyhow::Error> {
    let subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE confirmed = true
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::try_from(r.email) {
        Ok(email) => Ok(email),
        Err(err) => Err(anyhow::anyhow!(err)),
    })
    .collect();

    Ok(subscribers)
}
