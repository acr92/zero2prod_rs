use reqwest::Client;

use crate::harness::spawn_app;
use webserver::configuration::{get_configuration, pathbuf_relative_to_current_working_directory};
use webserver::startup::{get_connection_pool, migrate_sql, run};

pub mod harness;

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
