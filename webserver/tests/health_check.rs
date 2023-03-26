use crate::harness::spawn_app;
use reqwest::Client;
use testcontainers::clients::Cli;

pub mod harness;

#[actix_web::test]
async fn health_check_works() {
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    let client = Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
}
