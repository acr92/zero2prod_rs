use serde_json::{json, Value};
use testcontainers::clients::Cli;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::harness::{spawn_app, ConfirmationLinks, TestApp};

pub mod harness;

#[actix_web::test]
async fn post_newsletters_returns_400_if_missing_title() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;

    let payload = json!({
        "content": {
            "text": "Newsletter\
            This is my newsletter",
            "html": "<h1>Newsletter</h1><br />Hello, this is my newsletter!",
        }
    });

    // Act
    let response = app.post_newsletters(&payload).await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[actix_web::test]
async fn post_newsletters_returns_400_if_missing_content() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;

    let payload = json!({"title": "Newsletter"});

    // Act
    let response = app.post_newsletters(&payload).await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[actix_web::test]
async fn post_newsletters_does_not_send_email_to_unconfirmed_subscribers() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    let payload = payload();

    create_unconfirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_newsletters(&payload).await;

    // Assert
    let status = response.status().as_u16();
    println!("{}", response.text().await.unwrap());
    assert_eq!(200, status);
}

#[actix_web::test]
async fn post_newsletters_send_the_newsletter_to_all_confirmed_subscribers() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    let payload = payload();

    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_newsletters(&payload).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn post_newsletters_requires_authentication() {
    // Arrange
    let docker = Cli::default();
    let app = spawn_app(&docker).await;
    let payload = payload();

    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let response = reqwest::Client::new()
        .post(&format!("{}/admin/newsletters", &app.address))
        .json(&payload)
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(401, response.status().as_u16());
}

fn payload() -> Value {
    json!({
        "title": "Ursula Le Guin",
        "content": {
            "text": "Newsletter\
            This is my newsletter",
            "html": "<h1>Newsletter</h1><br />Hello, this is my newsletter!",
        }
    })
}

async fn create_unconfirmed_subscriber(app: &TestApp<'_>) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("create_unconfirmed_subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp<'_>) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.text).await.unwrap();
}
