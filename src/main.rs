use std::net::TcpListener;

use webserver::configuration::{
    get_configuration, pathbuf_relative_to_current_working_directory, Settings,
};
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
        .email
        .sender_email
        .clone()
        .try_into()
        .expect("Invalid email configuration");
    let timeout = config.email.timeout();
    let email_client = EmailClient::new(
        config.email.base_url.clone(),
        sender_email,
        config.email.authorization_token.clone(),
        timeout,
    );

    let base_url = get_base_url(config);

    run(tcp_listener, connection_pool, email_client, base_url)
        .await
        .unwrap()
        .await
        .unwrap();

    Ok(())
}

fn get_base_url(config: Settings) -> String {
    if matches!(config.application.port, 80 | 443) {
        config.application.base_url
    } else {
        format!(
            "{}:{}",
            config.application.base_url, config.application.port
        )
    }
}
