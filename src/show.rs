use clap::{error::ErrorKind, ArgMatches, Args, Command, Error as ClapError, FromArgMatches};
use thiserror::Error;

use crate::config::{Config, DbConfig};

#[derive(Error, Debug)]
pub enum ShowError {
    #[error("Not found")]
    NotFound,
}

pub struct Show {
    db_config: DbConfig,
}

impl FromArgMatches for Show {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, ClapError> {
        let mut matches = matches.clone();

        Self::from_arg_matches_mut(&mut matches)
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, ClapError> {
        let db_config_result = if let Some((name, _)) = matches.subcommand() {
            Config::new()
                .unwrap_or_default()
                .list()
                .into_iter()
                .find(|db_config| db_config.name == name)
        } else {
            None
        };
        let db_config = db_config_result
            .ok_or(ClapError::new(ErrorKind::ValueValidation))?
            .clone();

        Ok(Self { db_config })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), ClapError> {
        let mut matches = matches.clone();

        self.update_from_arg_matches_mut(&mut matches)
    }
}

impl Args for Show {
    fn augment_args(cmd: Command) -> Command {
        let config = Config::new().unwrap_or_default();
        let mut new_cmd = cmd.subcommand_required(true);

        for db_config in config.list() {
            if db_config.url.is_none() {
                continue;
            }

            new_cmd = new_cmd.subcommand(
                Command::new(db_config.name)
                    .about(db_config.description.unwrap_or("Database".to_string())),
            );
        }

        new_cmd
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Show::augment_args(cmd)
    }
}

impl Show {
    fn path(&self) -> Result<String, ShowError> {
        let url = self.db_config.url.as_ref().ok_or(ShowError::NotFound)?;

        Ok(if url.scheme() == "file" {
            url.path().to_string()
        } else {
            url.to_string()
        })
    }

    pub fn run(&self) -> Result<(), ShowError> {
        println!("{}", self.path()?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_show_run_postgresql() -> Result<(), ShowError> {
        let show = Show {
            db_config: DbConfig {
                name: "test".to_string(),
                url: Some(Url::parse("postgresql://localhost/test").unwrap()),
                r#type: None,
                description: None,
            },
        };

        assert_eq!(show.path()?, "postgresql://localhost/test");
        Ok(())
    }

    #[test]
    fn test_show_run_sqlite_file() -> Result<(), ShowError> {
        let show = Show {
            db_config: DbConfig {
                name: "sqlite".to_string(),
                url: Some(Url::parse("file:test.db").unwrap()),
                r#type: None,
                description: None,
            },
        };

        assert_eq!(show.path()?, "/test.db");
        Ok(())
    }

    #[test]
    fn test_show_run_no_url_error() {
        let show = Show {
            db_config: DbConfig {
                name: "test".to_string(),
                url: None,
                r#type: None,
                description: None,
            },
        };
        let result = show.run();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShowError::NotFound));
    }
}
