use super::config::Config;
use arc_swap::ArcSwap;
use axum::extract::FromRef;
use dashmap::{
    DashMap, Entry,
    iter::Iter,
    mapref::{multiple::RefMulti, one::Ref},
};
use message::{
    ChatMessage,
    codec::{ActiveCodec, CodecType},
    session::{Session, SessionCount},
    time::Created,
};
use serde_json::{Map, Value};
use std::{fmt::Debug, ops::Deref, sync::Arc};
use time::OffsetDateTime;
use tokio::sync::{RwLock, mpsc::UnboundedSender};

/// Encode a value and wrap it in the appropriate WS frame type (Text/JSON vs Binary/CBOR).
pub fn encode_ws<T: serde::Serialize>(codec: CodecType, value: &T) -> Option<axum::extract::ws::Message> {
    let bytes = ActiveCodec::new(codec).encode(value).ok()?;
    Some(match codec {
        CodecType::Json => {
            let s = String::from_utf8(bytes).unwrap_or_default();
            axum::extract::ws::Message::Text(s.into())
        }
        CodecType::Cbor => axum::extract::ws::Message::Binary(bytes.into()),
    })
}

#[derive(Clone, Debug)]
pub struct SessionManager<T> {
    map: DashMap<Session, T>,
}

impl<'a, T> IntoIterator for &'a SessionManager<T> {
    type Item = RefMulti<'a, Session, T>;
    type IntoIter = Iter<'a, Session, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<T> SessionManager<T> {
    fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn get(&self, k: &Session) -> Option<Ref<'_, Session, T>> {
        self.map.get(k)
    }

    pub fn insert(&mut self, k: Session, v: T) -> Option<T> {
        self.map.insert(k, v)
    }

    pub fn remove(&self, k: &Session) -> Option<(Session, T)> {
        self.map.remove(k)
    }

    pub fn contains_key(&self, k: &Session) -> bool {
        self.map.contains_key(k)
    }

    pub fn entry(&self, k: Session) -> Entry<'_, Session, T> {
        self.map.entry(k)
    }

    /// Returns a mutable reference to the value for the given key.
    /// Returns `None` if the key is not present.
    pub fn get_mut(&self, k: &Session) -> Option<dashmap::mapref::one::RefMut<'_, Session, T>> {
        self.map.get_mut(k)
    }
}

#[derive(Debug, Clone)]
pub struct Shared<T> {
    pub session: Arc<SessionManager<T>>,
    pub count: Arw<SessionCount>,
    pub config: Arc<ArcSwap<Config>>,
}

impl<T: Clone> FromRef<Shared<T>> for Arc<SessionManager<T>> {
    fn from_ref(input: &Shared<T>) -> Self {
        input.session.clone()
    }
}

impl<T> FromRef<Shared<T>> for Arw<SessionCount> {
    fn from_ref(input: &Shared<T>) -> Self {
        input.count.clone()
    }
}

impl<T> FromRef<Shared<T>> for Arc<ArcSwap<Config>> {
    fn from_ref(input: &Shared<T>) -> Self {
        input.config.clone()
    }
}

impl<T> Shared<T> {
    pub fn new(config: Arc<ArcSwap<Config>>) -> Self {
        Shared {
            session: Arc::new(SessionManager::new()),
            count: Arc::new(RwLock::new(SessionCount::default())),
            config,
        }
    }
}

pub type Info = Map<String, Value>;

#[derive(Debug, Clone)]
pub struct Client<T> {
    pub sender: T,
    pub term: tokio::sync::mpsc::Sender<bool>,
    pub created: OffsetDateTime,
    pub info: Info,
    /// Codec determined at handshake from URL query parameter.
    pub codec: CodecType,
}

impl<T> Deref for Client<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

pub type Sender = UnboundedSender<ChatMessage<Created>>;

pub type Arw<T> = Arc<RwLock<T>>;
pub type Asession<T> = Arc<SessionManager<Client<T>>>;
pub type StateChat<T> = Shared<Client<T>>;
