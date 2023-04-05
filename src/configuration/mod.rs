use config::Config;
use std::str::FromStr;

mod environment;
pub mod settings;

use self::environment::ConfigurationEnvironment;
pub use settings::*;

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Get current directory path
    let base_path = std::env::current_dir().expect("Failed to get current directory.");
    // Get the configuration directory
    let configuration_directory = base_path.join("configuration");
    // Get the current environment
    let environment = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".into());
    let environment = ConfigurationEnvironment::from_str(environment.as_str())
        .expect("Invalid APP_ENVIRONMENT variable value.");

    let config = Config::builder()
        // Add default base configuration
        .add_source(config::File::from(configuration_directory.join("base.yaml")).required(true))
        // add environment configuration
        .add_source(
            config::File::from(
                configuration_directory.join(format!("{}.yaml", environment.as_str())),
            )
            .required(true),
        )
        // Add Runtime environment configuration
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("_"),
        )
        .build()?;
    let settings = config.try_deserialize::<Settings>()?;
    Ok(settings)
}
