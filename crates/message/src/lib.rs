use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

pub mod session;
use session::Session;
pub mod config;
pub mod instance;
pub mod kafka;
pub mod queue;
pub mod time;

pub trait Event<C> {
    fn event(&self) -> Option<&str>;
    fn set_time(&mut self, time: C);
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Envelope<C> {
    pub receiver: Vec<Session>,
    #[serde(flatten)]
    pub message: ChatMessage<C>,
}

impl<C> Event<C> for Envelope<C> {
    fn event(&self) -> Option<&str> {
        self.message.event()
    }
    fn set_time(&mut self, time: C) {
        self.message.set_time(time);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ChatMessage<C> {
    pub sender: Session,
    pub created: Option<C>,
    pub content: Value,
}

impl<C> From<(Session, Value)> for ChatMessage<C>
where
    C: Default,
{
    fn from(value: (Session, Value)) -> Self {
        ChatMessage {
            sender: value.0,
            created: Some(C::default()),
            content: value.1,
        }
    }
}

fn get_value_event(v: &Value) -> Option<&str> {
    if v.is_object()
        && let Some(m) = v.as_object()
    {
        let r = m.get("event").and_then(|x| x.as_str());
        return r;
    };
    // TODO: For now, messages coming from the UI are not batched, and perhaps never will be. We will not process them in batches at this time.
    // SEE: gateway/src/libs/websocket.rs:129
    tracing::info!("get_value_event failed: {:?}", v);
    None
}

impl<C> Event<C> for ChatMessage<C> {
    fn event(&self) -> Option<&str> {
        get_value_event(&self.content)
    }

    fn set_time(&mut self, time: C) {
        self.created = Some(time);
    }
}
