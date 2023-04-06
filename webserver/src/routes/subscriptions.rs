use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use actix_web_codegen::post;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email::EmailClient;
use crate::startup::ApplicationBaseUrl;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryInto<NewSubscriber> for FormData {
    type Error = String;

    fn try_into(self) -> Result<NewSubscriber, Self::Error> {
        let email = SubscriberEmail::try_from(self.email)?;
        let name = SubscriberName::try_from(self.name)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[post("/subscriptions")]
pub async fn subscriptions(
    form: web::Form<FormData>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
    base_url: Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber, &base_url.0)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, subscriber)
)]
async fn insert_subscriber(pool: &PgPool, subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to the new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
async fn send_confirmation_email(
    email_client: &Data<EmailClient>,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token=mytoken",
        base_url
    );
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\n\
                Visit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await?;

    Ok(())
}
