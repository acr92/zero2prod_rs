use crate::harness::spawn_app;
use reqwest::Client;

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
