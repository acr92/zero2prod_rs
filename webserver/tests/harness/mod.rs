use crate::harness::postgres::Postgres;
use fake::{Fake, Faker};
use secrecy::Secret;
use sqlx::PgPool;
use std::net::TcpListener;
use std::time::Duration;
use testcontainers::clients::Cli;
use testcontainers::Container;
use webserver::configuration::{get_configuration, pathbuf_relative_to_current_working_directory};
use webserver::email::EmailClient;
use webserver::startup::{get_connection_pool, migrate_sql, run};

mod postgres;

pub struct TestApp<'b> {
    pub address: String,
    pub db_pool: PgPool,
    pub postgres_container: Container<'b, Postgres>,
}

pub async fn spawn_app(docker: &Cli) -> TestApp {
    let configuration_path =
        pathbuf_relative_to_current_working_directory(vec!["..", "configuration"]);
    let mut config = get_configuration(configuration_path).unwrap();

    let container = docker.run::<Postgres>(Postgres::default());
    config.database.port = container.get_host_port_ipv4(5432);
    config.database.database_name = String::from("postgres");

    let pool = get_connection_pool(&config.database).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let connection_pool = get_connection_pool(&config.database).await.unwrap();
    migrate_sql(&connection_pool).await.unwrap();

    let email_client = EmailClient::new(
        config.email_client.base_url,
        config
            .email_client
            .sender_email
            .try_into()
            .expect("Invalid email configuration"),
        Secret::new(Faker.fake()),
        Duration::from_secs(config.email_client.timeout_seconds),
    );

    let server = run(listener, connection_pool, email_client)
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: pool,
        postgres_container: container,
    }
}
