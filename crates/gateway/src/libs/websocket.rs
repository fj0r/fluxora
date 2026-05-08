use super::config::{Config, Hook};
use super::shared::{encode_ws, Client, StateChat};
use super::template::Tmpls;
use message::codec::{ActiveCodec, CodecType};
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
    codec: ActiveCodec,
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

    // Codec fixed at handshake time
    let codec = codec.as_type();
    tracing::info!("WS codec for {}: {:?}", &session.id, codec);

    let new_client = Client {
        sender: tx.clone(),
        term: term_tx.clone(),
        info: session.info.clone(),
        created: OffsetDateTime::now_utc(),
        codec,
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

    // Greet: send immediately using the codec determined from URL query parameter
    if let Some(greets) = config_reader.hooks.get("greet") {
        for g in greets.iter() {
            match g.greet::<T>(&context, tmpls.clone()).await {
                Ok(payload) => {
                    if let Some(ws_msg) = encode_ws(codec, &payload) {
                        let _ = sender.send(ws_msg).await;
                    }
                }
                Err(e) => {
                    tracing::error!("GreetError => {:?}", e);
                }
            }
        }
    }

    let sid = session.id.clone();
    let hooks = config_reader.hooks.clone();
    drop(config_reader);

    let mut recv_task = tokio::spawn(async move {
        #[allow(unused_mut)]
        let mut sid = sid;

        while let Some(Ok(msg)) = receiver.next().await {
            let value = match msg {
                axum::extract::ws::Message::Text(t) => {
                    match serde_json::from_str::<serde_json::Value>(&t) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("JSON decode: {:?}", e);
                            continue;
                        }
                    }
                }
                axum::extract::ws::Message::Binary(b) => {
                    let mut cursor = std::io::Cursor::new(&b);
                    match ciborium::de::from_reader::<serde_json::Value, _>(&mut cursor) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("CBOR decode: {:?}", e);
                            continue;
                        }
                    }
                }
                _ => continue,
            };

            let chat_msg: T = (sid.clone(), value).into();

            if let Some(ev) = chat_msg.event()
                && hooks.contains_key(ev)
                && let Some(wh) = hooks.get(ev)
            {
                for h in wh {
                    if h.disable { continue; }
                    match h.variant.handle(to_value(&chat_msg)?).await {
                        Ok(r) => {
                            let _ = tx.send((sid.clone(), r).into());
                        }
                        Err(e) => {
                            let mut err_ctx = Map::new();
                            err_ctx.insert("event".into(), ev.into());
                            err_ctx.insert("error".into(), e.to_string().into());
                            if let Ok(t) = tmpls.get_template("webhook_error.json")?.render(&err_ctx) {
                                let _ = tx.send(serde_json::from_str(&t)?);
                            }
                        }
                    }
                }
            } else {
                let _ = outgo_tx.send(chat_msg.clone());
            }

            tracing::debug!("[ws] {:?}", &chat_msg);
        }
        Okk(())
    });

    let codec_for_send = codec;
    let replaced = Arc::new(Mutex::new(false));
    let r1 = replaced.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    if let Some(ws_msg) = encode_ws(codec_for_send, &msg) {
                        if sender.send(ws_msg).await.is_err() {
                            break;
                        }
                    }
                },
                Some(_) = term_rx.recv() => { break; },
                else => {}
            }
        }
        *r1.lock().await = true;
        let _ = sender.close().await;
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
                        let _ = shared.session.get(&r).map(|c| c.send(x.message.clone()));
                    }
                }
            }
        }
        Some(())
    });
}
