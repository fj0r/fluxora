use figment::{
    Figment, Result,
    providers::{Env, Format, Toml},
};
use indexmap::IndexMap;
use message::kafka::config::Queue;
use notify::{Event, RecursiveMode, Result as ResultN, Watcher, recommended_watcher};
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, mpsc::channel};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum HookVariant {
    Path {
        path: String,
    },
    Webhook {
        endpoint: String,
        #[serde(default = "default_accept")]
        accept: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(unused)]
pub struct Hook {
    #[serde(default)]
    pub disable: bool,
    #[serde(flatten)]
    pub variant: HookVariant,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hooks(#[serde_as(as = "OneOrMany<_>")] pub Vec<Hook>);

impl Deref for Hooks {
    type Target = Vec<Hook>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a Hooks {
    type Item = &'a Hook;
    type IntoIter = std::slice::Iter<'a, Hook>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

fn default_accept() -> String {
    "application/json".to_owned()
}

pub type HookMap = IndexMap<String, Hooks>;

pub const ASSETS_PATH: &str = "manifest";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum LogFormat {
    #[allow(non_camel_case_types)]
    json,
    #[default]
    #[allow(non_camel_case_types)]
    compact,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    pub format: LogFormat,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub queue: Queue,
    pub hooks: HookMap,
    pub trace: Log,
    pub login_with_cookie: bool,
}

impl Config {
    pub fn new() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("gateway.toml"))
            .merge(Env::prefixed("GATEWAY_").split("_"))
            .extract()
    }
}

pub struct LiveConfig {
    pub data: Arc<Mutex<Config>>,
}

impl LiveConfig {
    pub fn new() -> Result<Self> {
        let x = Config::new()?;
        Ok(Self {
            data: Arc::new(Mutex::new(x)),
        })
    }

    #[allow(dead_code)]
    pub async fn listen(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = channel::<ResultN<Event>>();
        let mut watcher = recommended_watcher(tx)?;
        watcher.watch(Path::new("config.toml"), RecursiveMode::Recursive)?;
        let d = self.data.clone();
        tokio::task::spawn_blocking(|| async move {
            while let Ok(res) = rx.recv() {
                if res?.kind.is_modify() {
                    let n = Config::new()?;
                    dbg!("config update: {:?}", &n);
                    let mut x = d.lock().await;
                    *x = n;
                }
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        });
        Ok(())
    }
}
