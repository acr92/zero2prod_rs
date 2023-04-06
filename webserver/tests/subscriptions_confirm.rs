use testcontainers::clients::Cli;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::harness::spawn_app;

pub mod harness;

#[actix_web::test]
async fn subscriptions_without_token_are_rejected_with_a_400() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;

    // Act
    let response = reqwest::Client::new()
        .get(&format!("{}/subscriptions/confirm", &app.address))
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[actix_web::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions("name=le%20guin&email=ursula_le_guin%40gmail.com".into())
        .await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);
    assert_eq!(confirmation_links.text.host_str().unwrap(), "127.0.0.1");

    // Act
    let response = reqwest::get(confirmation_links.text).await.unwrap();

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions("name=le%20guin&email=ursula_le_guin%40gmail.com".into())
        .await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // Act
    let _ = reqwest::get(confirmation_links.text).await.unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, confirmed FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert!(saved.confirmed);
}
