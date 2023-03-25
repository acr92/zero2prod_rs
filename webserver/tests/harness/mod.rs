use crate::harness::postgres::Postgres;
use sqlx::PgPool;
use std::net::TcpListener;
use testcontainers::clients::Cli;
use webserver::configuration::{get_configuration, pathbuf_relative_to_current_working_directory};
use webserver::startup::{get_connection_pool, migrate_sql, run};

mod postgres;

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> Result<TestApp, String> {
    let configuration_path =
        pathbuf_relative_to_current_working_directory(vec!["..", "configuration"]);
    let mut config = get_configuration(configuration_path).map_err(|a| format!("{:#?}", a))?;

    let docker = Cli::default();
    let container = docker.run(Postgres::default());
    config.database.port = container.get_host_port_ipv4(5432);
    config.database.database_name = String::from("postgres");

    let pool = get_connection_pool(&config.database)
        .await
        .map_err(|a| format!("{:#?}", a))?;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let connection_pool = get_connection_pool(&config.database).await.unwrap();
    migrate_sql(&connection_pool).await.unwrap();

    let server = run(listener, connection_pool)
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    Ok(TestApp { address, pool })
}
