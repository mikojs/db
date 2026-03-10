use clap::Args;
use serde::Serialize;
use serde_json::Error as serdeJsonError;
use thiserror::Error;

use crate::config::{Config, ConfigError, DbConfig, DbType};

#[derive(Error, Debug)]
pub enum SqlsError {
    #[error("ConfigError: {0}")]
    Config(#[from] ConfigError),
    #[error("SerdeJsonError: {0}")]
    SerdeJson(#[from] serdeJsonError),
    #[error("Url Not Found")]
    UrlNotFound,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SqlsDbConfig {
    driver: DbType,
    data_source_name: String,
}

impl TryFrom<DbConfig> for SqlsDbConfig {
    type Error = SqlsError;

    fn try_from(value: DbConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            driver: value.r#type.unwrap_or(DbType::Postgresql),
            data_source_name: value
                .url
                .map(|url| url.to_string())
                .ok_or(SqlsError::UrlNotFound)?,
        })
    }
}

#[derive(Serialize)]
pub struct SqlsConfig(Vec<SqlsDbConfig>);

impl TryFrom<Config> for SqlsConfig {
    type Error = SqlsError;

    fn try_from(value: Config) -> Result<Self, Self::Error> {
        let mut sqls_config: Vec<SqlsDbConfig> = Vec::new();

        for db_config in value.list() {
            if db_config.url.is_some() {
                sqls_config.push(db_config.try_into()?);
            }
        }

        Ok(Self(sqls_config))
    }
}

#[derive(Args)]
pub struct Sqls {}

impl Sqls {
    pub fn run(&self) -> Result<(), SqlsError> {
        let config = Config::new()?;
        let sqls_config: SqlsConfig = config.try_into()?;

        println!("{}", serde_json::to_string_pretty(&sqls_config)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use url::Url;

    #[test]
    fn test_sqls_db_config_from_db_config_postgresql() -> Result<(), SqlsError> {
        let db_config = DbConfig {
            name: "test".to_string(),
            url: Some(Url::parse("postgresql://localhost/test").unwrap()),
            r#type: None,
            description: None,
        };
        let sqls_config: SqlsDbConfig = db_config.try_into()?;

        assert_eq!(sqls_config.driver, DbType::Postgresql);
        assert_eq!(sqls_config.data_source_name, "postgresql://localhost/test");
        assert_eq!(
            serde_json::to_string(&sqls_config)?,
            "{\"driver\":\"postgresql\",\"dataSourceName\":\"postgresql://localhost/test\"}"
        );
        Ok(())
    }

    #[test]
    fn test_sqls_db_config_from_db_config_sqlite() -> Result<(), SqlsError> {
        let db_config = DbConfig {
            name: "sqlite".to_string(),
            url: Some(Url::parse("file:test.db").unwrap()),
            r#type: Some(DbType::Sqlite),
            description: None,
        };
        let sqls_config: SqlsDbConfig = db_config.try_into()?;

        assert_eq!(sqls_config.driver, DbType::Sqlite);
        assert_eq!(sqls_config.data_source_name, "file:///test.db");
        Ok(())
    }

    #[test]
    fn test_sqls_db_config_no_url_error() {
        let db_config = DbConfig {
            name: "test".to_string(),
            url: None,
            r#type: Some(DbType::Postgresql),
            description: None,
        };
        let result: Result<SqlsDbConfig, SqlsError> = db_config.try_into();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SqlsError::UrlNotFound));
    }

    #[test]
    fn test_sqls_config_from_config_skips_no_url() -> Result<(), SqlsError> {
        env::set_var("DB_SQLSTEST_URL", "postgresql://localhost/test");
        env::set_var("DB_SQLSTEST_TYPE", "postgresql");

        let config = Config::new()?;
        let sqls_config: SqlsConfig = config.try_into()?;
        let has_valid = sqls_config
            .0
            .iter()
            .any(|c| c.data_source_name == "postgresql://localhost/test");

        assert!(has_valid);
        Ok(())
    }
}
