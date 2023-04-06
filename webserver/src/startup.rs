use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use sqlx::migrate::MigrateError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};
use tracing_actix_web::TracingLogger;

use crate::configuration::DatabaseSettings;
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
) -> Result<Server, Error> {
    let pool = Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::hello)
            .service(routes::subscriptions)
            .service(routes::subscriptions_confirm)
            .app_data(pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
