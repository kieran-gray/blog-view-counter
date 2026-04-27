use worker::Env;

use crate::api_worker::setup::exceptions::SetupError;

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub allowed_paths: Vec<String>,
}

impl Config {
    pub fn from_env(env: &Env) -> Result<Self, SetupError> {
        let allowed_origins = Config::parse_csv(env, "ALLOWED_ORIGINS")?;
        let allowed_paths = Config::parse_csv(env, "ALLOWED_PATHS")?;
        Ok(Self {
            allowed_origins,
            allowed_paths,
        })
    }

    fn parse_csv(env: &Env, var: &str) -> Result<Vec<String>, SetupError> {
        let env_var = env
            .var(var)
            .map_err(|_| SetupError::MissingVariable(var.to_string()))?
            .to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(env_var)
    }
}
