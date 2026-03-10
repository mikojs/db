use std::{env, str::FromStr};

use regex::{Error as RegexError, Regex};
use serde::Serialize;
use strum::ParseError as StrumParseError;
use strum_macros::EnumString;
use thiserror::Error;
use url::{ParseError as UrlParseError, Url};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("RegexError: {0}")]
    Regex(#[from] RegexError),
    #[error("ParseError: {0}")]
    StrumParse(#[from] StrumParseError),
    #[error("UrlParseError: {0}")]
    UrlParse(#[from] UrlParseError),
}

#[derive(EnumString, Serialize, Debug, PartialEq, Clone)]
pub enum DbType {
    #[serde(rename = "postgresql")]
    #[strum(serialize = "postgresql")]
    Postgresql,
    #[serde(rename = "sqlite3")]
    #[strum(serialize = "sqlite3")]
    Sqlite,
}

#[derive(Default, Clone)]
pub struct DbConfig {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<Url>,
    pub r#type: Option<DbType>,
}

#[derive(Default)]
pub struct Config(Vec<DbConfig>);

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut db_configs: Vec<DbConfig> = Vec::new();
        let db_parttern = Regex::new(r"^DB_(?<name>\w+)_(?<type>URL|TYPE|DESCRIPTION)$")?;

        for (key, value) in env::vars() {
            if value.is_empty() {
                continue;
            }

            if let Some(caps) = db_parttern.captures(&key) {
                let name = caps["name"].replace("_", "-").to_lowercase();
                let r#type = caps["type"].to_string();

                if let Some(index) = db_configs.iter().position(|c| c.name == name) {
                    Config::update_db_config_with_type(&mut db_configs[index], r#type, value)?;
                } else {
                    let mut db_config = DbConfig {
                        name,
                        ..Default::default()
                    };

                    Config::update_db_config_with_type(&mut db_config, r#type, value)?;
                    db_configs.push(db_config);
                }
            };
        }

        Ok(Self(db_configs))
    }

    fn update_db_config_with_type(
        db_config: &mut DbConfig,
        r#type: String,
        value: String,
    ) -> Result<(), ConfigError> {
        match r#type.as_ref() {
            "URL" => db_config.url = Some(Url::parse(&value)?),
            "TYPE" => db_config.r#type = Some(DbType::from_str(&value)?),
            "DESCRIPTION" => db_config.description = Some(value),
            _ => unreachable!("unknown type {}", r#type),
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<DbConfig> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const DB_DEFAULT_URL: &str = "postgresql://postgres:postgres@localhost/postgres";
    const DB_SQLITE_URL: &str = "file:foo.db";

    #[test]
    fn test_update_db_config_with_url() -> Result<(), ConfigError> {
        let mut config = DbConfig::default();

        Config::update_db_config_with_type(
            &mut config,
            "URL".to_string(),
            DB_DEFAULT_URL.to_string(),
        )?;
        assert_eq!(config.url, Some(Url::parse(DB_DEFAULT_URL)?));
        Ok(())
    }

    #[test]
    fn test_update_db_config_with_type() -> Result<(), ConfigError> {
        let mut config = DbConfig::default();

        Config::update_db_config_with_type(
            &mut config,
            "TYPE".to_string(),
            "postgresql".to_string(),
        )?;
        assert_eq!(config.r#type, Some(DbType::Postgresql));
        Ok(())
    }

    #[test]
    fn test_update_db_config_with_description() -> Result<(), ConfigError> {
        let mut config = DbConfig::default();

        Config::update_db_config_with_type(
            &mut config,
            "DESCRIPTION".to_string(),
            "test description".to_string(),
        )?;
        assert_eq!(config.description, Some("test description".to_string()));
        Ok(())
    }

    #[test]
    fn test_update_db_config_invalid_url() {
        let mut config = DbConfig::default();
        let result = Config::update_db_config_with_type(
            &mut config,
            "URL".to_string(),
            "not a valid url".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_update_db_config_invalid_type() {
        let mut config = DbConfig::default();
        let result = Config::update_db_config_with_type(
            &mut config,
            "TYPE".to_string(),
            "invalid_db_type".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn get_config() -> Result<(), ConfigError> {
        env::set_var("DB_DEFAULT_URL", DB_DEFAULT_URL);
        env::set_var("DB_TEST_TEST_URL", DB_DEFAULT_URL);
        env::set_var("DB_TEST_TEST_TYPE", "postgresql");
        env::set_var("DB_TEST_TEST_DESCRIPTION", "description");
        env::set_var("DB_SQLITE_URL", DB_SQLITE_URL);
        env::set_var("DB_SQLITE_TYPE", "sqlite3");
        env::set_var("DB_EMPTY_URL", "");

        let mut tested_config = HashMap::new();

        for config in Config::new()?.list() {
            tested_config.insert(config.name.clone(), true);
            match config.name.as_str() {
                "default" => assert_eq!(config.url, Some(Url::parse(DB_DEFAULT_URL)?)),
                "test-test" => {
                    assert_eq!(config.url, Some(Url::parse(DB_DEFAULT_URL)?));
                    assert_eq!(config.r#type, Some(DbType::Postgresql));
                    assert_eq!(config.description, Some("description".to_string()));
                }
                "sqlite" => {
                    assert_eq!(config.url, Some(Url::parse(DB_SQLITE_URL)?));
                    assert_eq!(config.r#type, Some(DbType::Sqlite));
                }
                _ => {}
            }
        }

        assert!(tested_config.len() >= 3);
        Ok(())
    }

    #[test]
    fn test_name_underscore_to_hyphen() -> Result<(), ConfigError> {
        env::set_var("DB_MY_DATABASE_NAME_URL", "postgresql://localhost/test");

        let config = Config::new()?;
        let db = config
            .list()
            .into_iter()
            .find(|c| c.name == "my-database-name");

        assert!(db.is_some());
        Ok(())
    }

    #[test]
    fn test_name_lowercase() -> Result<(), ConfigError> {
        env::set_var("DB_UPPERCASE_URL", "postgresql://localhost/test");

        let config = Config::new()?;
        let db = config.list().into_iter().find(|c| c.name == "uppercase");

        assert!(db.is_some());
        Ok(())
    }
}
