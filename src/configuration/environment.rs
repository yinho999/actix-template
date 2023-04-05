use std::str::FromStr;

pub enum ConfigurationEnvironment {
    Development,
    Production,
}

impl ConfigurationEnvironment {
    pub fn as_str(&self) -> &str {
        match self {
            ConfigurationEnvironment::Development => "development",
            ConfigurationEnvironment::Production => "production",
        }
    }
}

impl FromStr for ConfigurationEnvironment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" => Ok(ConfigurationEnvironment::Development),
            "production" => Ok(ConfigurationEnvironment::Production),
            _ => Err(format!("'{}' is not a valid configuration environment. Use either development or production", s)),
        }
    }
}
