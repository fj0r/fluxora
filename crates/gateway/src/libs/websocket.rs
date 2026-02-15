use super::config::{Config, Hook};
use super::shared::{Client, StateChat};
use super::template::Tmpls;
use anyhow::{Ok as Okk, Result};
use arc_swap::ArcSwap;
use axum::extract::ws::WebSocket;
use dashmap::Entry;
use futures::{sink::SinkExt, stream::StreamExt};
use message::{
    Event,
    session::{Session, SessionInfo},
    time::Created,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, to_value};
use std::fmt::Debug;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

/* TODO:
use std::async_iter;

struct GreetIter<'a> {
    index: usize,
    greet: &'a AssetsList,
    context: &'a minijinja::Value,
}

impl<'a> AsyncIterator for GreetIter<'a> {
}
*/

impl Hook {
    async fn greet<T>(&self, context: &Map<String, Value>, tmpls: Arc<Tmpls<'_>>) -> Result<T>
    where
        T: Event<Created> + Serialize + From<(Session, Value)>,
    {
        let v = self.handle(context, tmpls).await?;
        let msg: T = (Session::default(), v).into();
        Ok(msg)
    }
}

pub async fn handle_ws<T>(
    socket: WebSocket,
    outgo_tx: UnboundedSender<T>,
    state: StateChat<UnboundedSender<T>>,
    config: Arc<ArcSwap<Config>>,
    tmpls: Arc<Tmpls<'static>>,
    session: &SessionInfo,
) where
    T: Event<Created>
        + for<'a> Deserialize<'a>
        + Serialize
        + From<(Session, Value)>
        + Clone
        + Debug
        + Send
        + 'static,
{
    let config_reader = config.load();
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();
    let (term_tx, mut term_rx) = tokio::sync::mpsc::channel(1);

    let new_client = Client {
        sender: tx.clone(),
        term: term_tx.clone(),
        info: session.info.clone(),
        created: OffsetDateTime::now_utc(),
    };
    {
        match state.session.entry(session.id.clone()) {
            Entry::Occupied(mut e) => {
                let g = e.get_mut();
                let _ = g.term.send(true).await;
                *g = new_client;
            }
            Entry::Vacant(e) => {
                e.insert(new_client);
            }
        }
    }

    tracing::info!("Connection opened for {}", &session.id);

    let mut context = Map::new();
    context.insert("session_id".into(), session.id.clone().into());
    context.insert("info".into(), Value::Object(session.info.clone()));

    if let Some(greet) = config_reader.hooks.get("greet") {
        for g in greet.iter() {
            match g.greet::<T>(&context, tmpls.clone()).await {
                Ok(payload) => {
                    if let Ok(text) = serde_json::to_string(&payload) {
                        let _ = sender
                            .send(axum::extract::ws::Message::Text(text.into()))
                            .await;
                    }
                }
                Err(e) => {
                    println!("GreetError => {:?}", e)
                }
            }
        }
    }

    let replaced = Arc::new(Mutex::new(false));
    let r1 = replaced.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    let text = serde_json::to_string(&msg)?;
                    // to ws client
                    if sender
                        .send(axum::extract::ws::Message::Text(text.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                },
                Some(_) = term_rx.recv() => {
                    break;
                },
                else => {}
            }
        }
        *r1.lock().await = true;
        let _ = sender.close().await;
        Okk(())
    });

    let sid_cloned = session.id.clone();
    let hooks = config_reader.hooks.clone();
    drop(config_reader); // release lock
    let mut recv_task = tokio::spawn(async move {
        #[allow(unused_mut)]
        let mut sid = sid_cloned;

        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            let text = msg.to_text()?;
            let value = serde_json::from_str(text)?;
            let chat_msg: T = (sid.clone(), value).into();

            // TODO: process messages in batches?
            if let Some(ev) = chat_msg.event()
                && hooks.contains_key(ev)
                && let Some(wh) = hooks.get(ev)
            {
                for h in wh {
                    if h.disable {
                        continue;
                    }
                    match h.variant.handle(to_value(&chat_msg)?).await {
                        Ok(r) => {
                            let _ = tx.send((sid.clone(), r).into());
                        }
                        Err(e) => {
                            context.insert("event".into(), ev.into());
                            context.insert("error".into(), e.to_string().into());
                            let t = tmpls.get_template("webhook_error.json")?.render(&context)?;
                            let _ = tx.send(serde_json::from_str(&t)?);
                        }
                    }
                }
            } else {
                // send to event MQ
                let _ = outgo_tx.send(chat_msg.clone());
            }

            tracing::debug!("[ws] {:?}", &chat_msg);
        }
        Okk(())
    });

    tokio::select! {
        _ = &mut recv_task => {
            let _ = term_tx.send(true).await;
        },
        _ = &mut send_task => send_task.abort(),
    };

    tracing::info!("Connection closed for {}", &session.id);
    if !*replaced.lock().await {
        tracing::info!("Remove session: {}", &session.id);
        state.session.remove(&session.id);
    };
}

use message::{ChatMessage, Envelope};

pub async fn send_to_ws(
    income_rx: Arc<Mutex<UnboundedReceiver<Envelope<Created>>>>,
    shared: &StateChat<UnboundedSender<ChatMessage<Created>>>,
) {
    let shared = shared.clone();
    tokio::spawn(async move {
        let mut rx = income_rx.lock().await;

        while let Some(x) = rx.recv().await {
            if !x.receiver.is_empty() {
                for r in x.receiver {
                    if shared.session.contains_key(&r) {
                        let s = shared.session.get(&r)?;
                        let _ = s.send(x.message.clone());
                    }
                }
            }
        }
        Some(())
    });
}
