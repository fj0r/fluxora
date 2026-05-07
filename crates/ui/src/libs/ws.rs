use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket;
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::futures::WebSocket;
use js_sys::wasm_bindgen::JsError;
use message::codec::{ActiveCodec, CodecType};

pub use gloo_net::websocket::Message;

#[derive(Clone, Copy)]
pub struct WebSocketHandle {
    ws_write: Signal<Rc<RefCell<SplitSink<WebSocket, Message>>>>,
    state: Signal<websocket::State>,
    message: Signal<String>,
}

/// Opens a web socket connection at the specified `url`.
/// Decodes incoming messages using the specified `codec_type`.
pub fn use_web_socket(url: &str, codec_type: CodecType) -> Result<WebSocketHandle, JsError> {
    let state = use_signal(|| websocket::State::Closed);
    let mut message = use_signal(String::new);
    let codec = ActiveCodec::new(codec_type);

    let ws = WebSocket::open(url)?;
    let (write, mut read) = ws.split();

    spawn(async move {
        while let Some(Ok(m)) = read.next().await {
            match m {
                Message::Text(t) => {
                    // Fallback for text if sent
                    message.set(t);
                }
                Message::Bytes(b) => {
                    if let Ok(decoded) = codec.decode::<String>(&b) {
                        message.set(decoded);
                    } else if let Ok(decoded) = String::from_utf8(b.clone()) {
                        message.set(decoded);
                    }
                }
            }
        }
    });

    Ok(WebSocketHandle {
        ws_write: use_signal(|| Rc::new(RefCell::new(write))),
        state,
        message,
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

    pub fn message_texts(self) -> Signal<String> {
        self.message
    }

    /// NOTE: Not yet implemented due to technical reasons.
    #[allow(unused)]
    pub fn close(self) {
        unimplemented!();
    }
}
