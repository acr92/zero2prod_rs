use actix_web::{App, HttpServer};
use sqlx::PgPool;
use webserver::configuration::get_configuration;
use webserver::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().unwrap();

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!().run(&connection_pool).await.unwrap();
    println!("Migrated");

    HttpServer::new(|| App::new().service(webserver::routes::hello))
        .bind(("127.0.0.1", config.application_port))?
        .run()
        .await
}
