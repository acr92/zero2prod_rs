use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use sqlx::migrate::MigrateError;
use sqlx::{Error, PgPool};

use crate::configuration::DatabaseSettings;

pub async fn get_connection_pool(database: &DatabaseSettings) -> Result<PgPool, Error> {
    PgPool::connect(database.connection_string().as_str()).await
}

pub async fn migrate_sql(connection_pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("../migrations").run(connection_pool).await
}

pub async fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, Error> {
    let pool = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(crate::routes::hello)
            .service(crate::routes::subscriptions)
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
