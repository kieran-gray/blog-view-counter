use anyhow::{Context, Result};
use worker::Env;

use crate::api_worker::{Config, api::client::DurableObjectClient};

pub struct AppState {
    pub config: Config,
    pub do_client: DurableObjectClient,
}

impl AppState {
    pub fn from_env(env: &Env) -> Result<Self> {
        let config = Config::from_env(env)?;
        let do_client = DurableObjectClient::new(
            env.durable_object("VIEW_COUNTER")
                .context("Missing binding VIEW_COUNTER")?,
        );

        Ok(Self { config, do_client })
    }
}
