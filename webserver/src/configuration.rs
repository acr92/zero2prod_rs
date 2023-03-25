use std::net::TcpListener;
use std::path::PathBuf;

use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub application_port: u16,
}

impl TryInto<TcpListener> for Settings {
    type Error = std::io::Error;

    fn try_into(self) -> Result<TcpListener, Self::Error> {
        TcpListener::bind(("127.0.0.1", self.application_port))
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name,
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        )
    }
}

pub fn pathbuf_relative_to_current_working_directory(path: Vec<&str>) -> PathBuf {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");

    let mut full_path = base_path;
    for part in path {
        full_path = full_path.join(part);
    }

    full_path
}

pub fn get_configuration(
    configuration_directory: PathBuf,
) -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
