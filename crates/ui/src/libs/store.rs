use super::ws::{WebSocketHandle, use_web_socket};
use anyhow::Result;
use brick::{
    Brick, BrickOps,
    merge::{BrickOp, Concat, Delete, Replace},
};
use content::{Content, Message, Method, Outflow};
#[allow(unused_imports)]
use dioxus::prelude::*;
use js_sys::wasm_bindgen::JsError;
use minijinja::Environment;
use serde_json::{Value, from_str, to_string, to_string_pretty};
use std::collections::HashMap;
use std::str;
use std::sync::{LazyLock, RwLock};

static TMPL: LazyLock<RwLock<Environment>> = LazyLock::new(|| {
    let env = Environment::new();
    //env.set_auto_escape_callback(|_| AutoEscape::Json);
    RwLock::new(env)
});

#[derive(Clone)]
pub struct Status {
    pub ws: WebSocketHandle,
    pub layout: Signal<Brick>,
    pub data: Signal<HashMap<String, Brick>>,
    pub list: Signal<HashMap<String, Vec<Brick>>>,
}

impl Status {
    pub async fn send(&mut self, event: impl AsRef<str>, id: Option<String>, content: Value) {
        let x = Outflow {
            event: event.as_ref().to_string(),
            id,
            data: content,
        };

        if let Ok(msg) = to_string::<Outflow>(&x) {
            let msg = gloo_net::websocket::Message::Text(msg);
            let _ = self.ws.send(msg).await;
        }
    }

    pub fn set(&mut self, name: impl AsRef<str>, brick: Brick) {
        self.data.write().insert(name.as_ref().to_string(), brick);
    }
}

fn dispatch(
    act: Message<Brick>,
    layout: &mut Signal<Brick>,
    data: &mut Signal<HashMap<String, Brick>>,
    list: &mut Signal<HashMap<String, Vec<Brick>>>,
) {
    let Message {
        sender: _,
        created: _,
        content,
    } = act;
    for c in content {
        match c {
            Content::Tmpl(x) => {
                let n = x.name;
                let d = x.data;
                let _ = TMPL
                    .write()
                    .expect("write TMPL failed")
                    .add_template_owned(n, d);
            }
            Content::Create(mut x) => {
                let env = TMPL.read().expect("read TMPL failed");
                x.data.render(&env);
                layout.set(x.data)
            }
            Content::Set(x) => {
                let e = x.event;
                let mut d = x.data;
                let env = TMPL.read().expect("read TMPL failed");
                d.render(&env);
                data.write().insert(e, d);
            }
            Content::Join(mut x) => {
                let env = TMPL.read().expect("read TMPL failed");
                x.data.render(&env);
                let e = x.event;
                let d = &mut x.data;
                let vs: &dyn BrickOp = match x.method {
                    Method::Replace => &Replace,
                    Method::Concat => &Concat,
                    Method::Delete => &Delete,
                };
                if let Some(_id) = &d.get_id() {
                    let mut l = list.write();
                    let list = l.entry(e).or_default();
                    let mut is_merge = false;
                    for i in list.iter_mut() {
                        if i.cmp_id(d) {
                            is_merge = true;
                            i.merge(vs, d);
                        }
                    }
                    if !is_merge {
                        list.push(d.clone());
                    }
                } else {
                    list.write().entry(e).or_default().push(d.clone());
                }
            }
            Content::Empty => {}
        }
    }
}

pub fn use_status(url: &str) -> Result<Status, JsError> {
    let ws = use_web_socket(url)?;
    let x = ws.message_texts();

    let mut layout = use_signal::<Brick>(|| {
        Brick::text(brick::Text {
            ..Default::default()
        })
    });
    let mut data = use_signal::<HashMap<String, Brick>>(HashMap::new);
    let mut list = use_signal::<HashMap<String, Vec<Brick>>>(HashMap::new);

    use_memo(move || {
        let act = &x();
        if !act.is_empty() {
            match from_str::<Message<Brick>>(act) {
                Ok(act) => dispatch(act, &mut layout, &mut data, &mut list),
                Err(err) => {
                    if let Ok(act) = &from_str::<serde_json::Value>(act)
                        && let Ok(act) = to_string_pretty(act)
                    {
                        dioxus::logger::tracing::info!(
                            "deserialize from_str error:\n {:#?}\n{}",
                            err,
                            act
                        )
                    }
                }
            }
        }
    });

    Ok(Status {
        ws,
        layout,
        data,
        list,
    })
}
