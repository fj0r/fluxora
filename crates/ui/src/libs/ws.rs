use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket;
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::futures::WebSocket;
use js_sys::wasm_bindgen::JsError;

pub use gloo_net::websocket::Message;

#[derive(Clone, Copy)]
pub struct WebSocketHandle {
    ws_write: Signal<Rc<RefCell<SplitSink<WebSocket, Message>>>>,
    state: Signal<websocket::State>,
    message_bytes: Signal<Vec<u8>>,
}

/// Opens a web socket connection at the specified `url`.
pub fn use_web_socket(url: &str) -> Result<WebSocketHandle, JsError> {
    let state = use_signal(|| websocket::State::Closed);
    let mut message_bytes = use_signal(Vec::new);

    let ws = WebSocket::open(url)?;
    let (write, mut read) = ws.split();

    spawn(async move {
        while let Some(Ok(m)) = read.next().await {
            match m {
                Message::Text(t) => message_bytes.set(t.into_bytes()),
                Message::Bytes(b) => message_bytes.set(b),
            }
        }
    });

    Ok(WebSocketHandle {
        ws_write: use_signal(|| Rc::new(RefCell::new(write))),
        state,
        message_bytes,
    })
}

impl WebSocketHandle {
    // TODO: solve this issue
    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn send(&mut self, message: Message) -> Result<(), WebSocketError> {
        self.ws_write.write().borrow_mut().send(message).await
    }

    #[allow(unused)]
    pub fn status(self) -> Signal<websocket::State> {
        self.state
    }

    pub fn message_bytes(self) -> Signal<Vec<u8>> {
        self.message_bytes
    }

    /// NOTE: Not yet implemented due to technical reasons.
    #[allow(unused)]
    pub fn close(self) {
        unimplemented!();
    }
}
