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

#[derive(Serialize)]
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
