use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize)]
pub struct Settings {
    // CHECK: I assume that this can be retrieved from DATABASE_URL environment variable
    // and not being duplicated here
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)            
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // CAUTION: You must run from src folder in order for this to work
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuring_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(configuring_directory.join("base.yaml")))
        .add_source(config::File::from(
            configuring_directory.join(environment_filename),
        ))
        .build()?;

    // Initialize our configuration reader
    // TODO: Try to parse DatabaseSettings from the environment variable
    // let settings = config::Config::builder()
    //     // Add configuration values from a file named `configuration.yaml`
    //     .add_source(config::File::new(
    //         "configuration.yaml",
    //         config::FileFormat::Yaml,
    //     ))
    // .build()?;
    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_deserialize::<Settings>()
}

/// The possible runtime environment for the application.
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`",
                other
            )),
        }
    }
}
