use crate::concat_fields;
use figment::{
    Figment, Result,
    providers::{Env, Format, Toml},
};
use indexmap::IndexMap;
use message::config::Queue;
use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};
use std::ops::Deref;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    #[serde(rename = "type")]
    pub kind: String,
    pub host: String,
    pub port: u16,
    pub db: String,
    pub schema: Option<String>,
    pub user: String,
    pub passwd: String,
}

impl Database {
    pub fn to_st(self: &Database) -> String {
        concat_fields! {
            var self;
            host;
            port;
            dbname = db;
            user;
            password = passwd;
        }
    }

    #[allow(unused)]
    pub fn to_url(self: &Database) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.passwd, self.host, self.port, self.db
        )
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum LogFormat {
    #[allow(non_camel_case_types)]
    json,
    #[default]
    #[allow(non_camel_case_types)]
    compact,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub format: LogFormat,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Logic {
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "crm")]
    Crm,
    #[serde(rename = "echo")]
    Echo,
}

fn default_accept() -> String {
    "application/json".to_owned()
}

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
        render: Option<String>,
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

pub type HookMap = IndexMap<String, Hooks>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Gateway {
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub logic: Logic,
    pub queue: Queue,
    pub database: Database,
    pub trace: Log,
    pub gateway: Gateway,
    pub hooks: HookMap,
}

impl Config {
    pub fn new() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("chat.toml"))
            .merge(Env::prefixed("CHAT_").split("_"))
            .extract()
    }
}

pub const ASSETS_PATH: &str = "manifest";
