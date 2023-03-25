use std::net::TcpListener;

use reqwest::Client;
use sqlx::PgPool;

use webserver::configuration::{get_configuration, pathbuf_relative_to_current_working_directory};
use webserver::startup::{get_connection_pool, migrate_sql, run};

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> Result<TestApp, String> {
    let configuration_path =
        pathbuf_relative_to_current_working_directory(vec!["..", "configuration"]);
    let config = get_configuration(configuration_path).map_err(|a| format!("{:#?}", a))?;
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

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await.unwrap();
    let client = Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}
