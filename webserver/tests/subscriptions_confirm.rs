use reqwest::Url;
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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(1, links.len());
        links[0].as_str().to_owned()
    };

    let raw_link = &get_link(body["HtmlBody"].as_str().unwrap());
    let confirmation_link = Url::parse(raw_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // Act
    let response = reqwest::get(confirmation_link).await.unwrap();

    // Assert
    assert_eq!(200, response.status().as_u16());
}
