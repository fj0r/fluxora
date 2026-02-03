use super::config::{Hook, HookVariant};
use super::template::Tmpls;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_str};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum HookError {
    #[error("reqwest error")]
    Reqwest(#[from] Error),
    #[error("not a webhook")]
    NotWebhook,
    #[error("render error")]
    Render(#[from] minijinja::Error),
    #[error("deserialize error")]
    JsonError(#[from] serde_json::Error),
    #[error("disabled")]
    Disabled,
}

impl Hook {
    pub async fn handle<T>(
        &self,
        msg: &Map<String, Value>,
        tmpls: Arc<Tmpls<'_>>,
    ) -> Result<T, HookError>
    where
        T: for<'de> Deserialize<'de> + Debug + Serialize,
    {
        if self.disable {
            return Err(HookError::Disabled);
        }

        match &self.variant {
            HookVariant::Path { path } => {
                let tmpl = tmpls.get_template(path).unwrap();
                let r = tmpl.render(msg)?;
                let r = from_str::<T>(&r)?;
                Ok(r)
            }
            HookVariant::Webhook {
                endpoint,
                render,
                accept: _,
            } => {
                let client = reqwest::Client::new();
                let r = client.post(endpoint).json(msg).send().await?;
                let mut r = r.json::<T>().await?;
                if let Some(path) = render {
                    // TODO: In the context of global state, non-file templates can be updated via the API.
                    let tmpl = tmpls.get_template(path).unwrap();
                    r = from_str::<T>(&tmpl.render(&r)?)?;
                };
                Ok(r)
            }
        }
    }
}

impl HookVariant {
    pub async fn handle<T>(&self, msg: T) -> Result<T, HookError>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de>,
    {
        if let HookVariant::Webhook {
            endpoint,
            accept: _,
            render: _,
        } = self
        {
            let client = reqwest::Client::new();
            let r = client.post(endpoint).json(&msg).send().await?;
            Ok(r.json::<T>().await?)
        } else {
            Err(HookError::NotWebhook)
        }
    }
}
