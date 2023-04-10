use actix_4_jwt_auth::biscuit::{Validation, ValidationOptions};
use actix_4_jwt_auth::{Oidc, OidcBiscuitValidator};
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::migrate::MigrateError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};
use tracing_actix_web::TracingLogger;

use crate::configuration::{AuthSettings, DatabaseSettings};
use crate::email::EmailClient;
use crate::routes;

pub async fn get_connection_pool(database: &DatabaseSettings) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(database.with_db().as_str())
        .await
}

pub async fn migrate_sql(connection_pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations").run(connection_pool).await
}

pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    auth_settings: AuthSettings,
) -> Result<Server, anyhow::Error> {
    let pool = Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let oidc = Oidc::new(auth_settings.clone().try_into()?).await.unwrap();

    let biscuit_validator = OidcBiscuitValidator {
        options: ValidationOptions {
            issuer: Validation::Validate(auth_settings.authority + "/"),
            ..ValidationOptions::default()
        },
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::hello)
            .service(routes::subscriptions)
            .service(routes::subscriptions_confirm)
            .service(
                web::scope("/admin")
                    .wrap(biscuit_validator.clone())
                    .service(routes::newsletters),
            )
            .app_data(oidc.clone())
            .app_data(pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
