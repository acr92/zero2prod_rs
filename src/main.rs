use std::net::TcpListener;
use webserver::configuration::{get_configuration, pathbuf_relative_to_current_working_directory};
use webserver::email::EmailClient;
use webserver::startup::{get_connection_pool, migrate_sql, run};
use webserver::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration_path = pathbuf_relative_to_current_working_directory(vec!["configuration"]);
    let config = get_configuration(configuration_path).unwrap();
    let tcp_listener: TcpListener = config.clone().try_into().unwrap();

    let connection_pool = get_connection_pool(&config.database).await.unwrap();
    migrate_sql(&connection_pool).await.unwrap();

    let sender_email = config
        .email_client
        .sender_email
        .clone()
        .try_into()
        .expect("Invalid email configuration");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token.clone(),
        timeout,
    );

    run(tcp_listener, connection_pool, email_client)
        .await
        .unwrap()
        .await
        .unwrap();

    Ok(())
}
