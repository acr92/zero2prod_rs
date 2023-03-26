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

pub async fn get_connection_pool(database: &DatabaseSettings) -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(database.with_db().as_str())
        .await
}

pub async fn migrate_sql(connection_pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations").run(connection_pool).await
}

pub async fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, Error> {
    let pool = Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(crate::routes::hello)
            .service(crate::routes::subscriptions)
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
