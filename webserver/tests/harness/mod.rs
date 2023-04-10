use std::net::TcpListener;
use std::time::Duration;

use fake::{Fake, Faker};
use reqwest::header::AUTHORIZATION;
use reqwest::Response;
use secrecy::Secret;
use serde_json::Value;
use sqlx::PgPool;
use testcontainers::clients::Cli;
use testcontainers::Container;
use wiremock::MockServer;

use crate::harness::jwks::{create_jwt_token, JWT_AUTHORITY};
use webserver::configuration::{
    get_configuration, pathbuf_relative_to_current_working_directory, AuthSettings,
};
use webserver::email::EmailClient;
use webserver::startup::{get_connection_pool, migrate_sql, run};

use crate::harness::postgres::Postgres;

mod jwks;
mod postgres;

pub struct TestApp<'b> {
    pub address: String,
    pub db_pool: PgPool,
    pub postgres_container: Container<'b, Postgres>,
    pub email_server: MockServer,
    pub port: u16,
}

pub async fn spawn_app(docker: &Cli) -> TestApp {
    let configuration_path =
        pathbuf_relative_to_current_working_directory(vec!["..", "configuration"]);
    let mut config = get_configuration(configuration_path).unwrap();

    let email_server = MockServer::start().await;
    config.email.base_url = email_server.uri();

    let container = docker.run::<Postgres>(Postgres::default());
    config.database.port = container.get_host_port_ipv4(5432);
    config.database.database_name = String::from("postgres");

    let pool = get_connection_pool(&config.database).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    config.application.base_url = address.clone();

    let connection_pool = get_connection_pool(&config.database).await.unwrap();
    migrate_sql(&connection_pool).await.unwrap();

    let email_client = EmailClient::new(
        config.email.base_url,
        config
            .email
            .sender_email
            .try_into()
            .expect("Invalid email configuration"),
        Secret::new(Faker.fake()),
        Duration::from_secs(config.email.timeout_seconds),
    );

    let auth_settings = AuthSettings {
        authority: JWT_AUTHORITY.to_string(),
        jwks: Some(include_str!("jwk_public_key.json").to_string()),
    };

    let server = run(
        listener,
        connection_pool,
        email_client,
        config.application.base_url,
        auth_settings,
    )
    .await
    .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        port,
        db_pool: pool,
        postgres_container: container,
        email_server,
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

impl TestApp<'_> {
    pub async fn post_subscriptions(&self, body: String) -> Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(1, links.len());
            let url = links[0].as_str().to_owned();
            reqwest::Url::parse(&url).unwrap()
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let text = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, text }
    }

    pub async fn post_newsletters(&self, payload: &Value) -> Response {
        reqwest::Client::new()
            .post(&format!("{}/admin/newsletters", &self.address))
            .json(&payload)
            .header(AUTHORIZATION, format!("Bearer {}", create_jwt_token()))
            .send()
            .await
            .unwrap()
    }
}
