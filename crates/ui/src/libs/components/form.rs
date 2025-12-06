use serde_json::{Value, to_value};
use std::collections::HashMap;

use super::super::store::Status;
use super::{Dynamic, Frame};
use brick::{Bind, BindVariant, Brick, BrickOps, Case, CaseAttr, Form, FormAttr, JsType};
use dioxus::prelude::*;
use maplit::hashmap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    pub data: Value,
    pub payload: Option<Value>,
}

type FormScope = HashMap<String, (Signal<Value>, Option<Value>)>;

fn walk(brick: &mut Brick, scope: &mut FormScope, confirm: Signal<Value>) {
    match brick.get_bind().and_then(|x| x.get("value")) {
        Some(Bind {
            default,
            r#type: kind,
            variant:
                BindVariant::Field {
                    field,
                    payload,
                    signal: _,
                },
        }) => {
            let kind = kind.clone();
            let v = match kind {
                Some(JsType::number) => {
                    let n = default
                        .as_ref()
                        .and_then(|x| x.as_f64())
                        .unwrap_or(0 as f64);
                    to_value(n).unwrap()
                }
                Some(JsType::bool) => {
                    let b = default.as_ref().and_then(|x| x.as_bool()).unwrap_or(false);
                    to_value(b).unwrap()
                }
                _ => {
                    let s = default.as_ref().and_then(|x| x.as_str()).unwrap_or("");
                    to_value(s).unwrap()
                }
            };

            let s = use_signal(|| v);
            scope.insert(field.to_string(), (s, payload.clone()));
            brick.set_bind(Some(hashmap! {
                "value".to_owned() => Bind {
                    r#type: kind,
                    default: None,
                    variant: BindVariant::Field {
                        field: field.to_string(),
                        payload: None,
                        signal: Some(s),
                    },
                },
            }));
        }
        Some(Bind {
            default: _,
            r#type: _,
            variant: BindVariant::Submit { .. },
        }) => {
            brick.set_bind(Some(hashmap! {
                "value".to_owned() => Bind {
                    variant: BindVariant::Submit {
                        submit: true,
                        signal: Some(confirm),
                    },
                    ..Default::default()
                },
            }));
        }
        _ => {}
    };
    if let Some(children) = &mut brick.borrow_sub_mut() {
        for c in children.iter_mut() {
            walk(c, scope, confirm);
        }
    };
}

#[component]
pub fn form_(id: Option<String>, brick: Form, children: Element) -> Element {
    // TODO: instant
    let _instant = if let Some(FormAttr {
        instant: Some(instant),
        ..
    }) = brick.attrs
    {
        instant
    } else {
        false
    };

    let mut data: FormScope = HashMap::new();
    let confirm = use_signal(|| Value::Bool(false));
    let mut brick = Brick::form(brick);
    walk(&mut brick, &mut data, confirm);
    let v = Vec::new();
    let children = brick.borrow_sub().unwrap_or(&v);
    let children = children.iter().map(|c| {
        rsx! {
            Frame { brick: c.clone() }
        }
    });

    let lc = brick.get_bind().and_then(|x| x.get("value")).cloned();
    if let Some(Bind {
        variant: BindVariant::Event { event },
        ..
    }) = lc
    {
        let s = use_context::<Status>();
        let content: HashMap<String, Message> = data
            .iter()
            .map(|(k, v)| {
                let d = Message {
                    data: v.0(),
                    payload: v.1.clone(),
                };
                (k.to_owned(), d)
            })
            .collect();
        //dioxus_logger::tracing::info!("{payload:?}");
        let v = to_value(content).unwrap();
        let _ = use_resource(move || {
            let ev = event.clone();
            let mut s = s.clone();
            let v = v.clone();
            async move {
                if let Some(c) = confirm.read().as_bool()
                    && c
                {
                    s.send(ev, None, v).await;
                }
            }
        });
    };

    if let Brick::form(Form {
        id, attrs, sub: c, ..
    }) = &brick
    {
        let brick = Brick::case(Case {
            id: id.clone(),
            attrs: attrs.as_ref().map(|FormAttr { class, .. }| CaseAttr {
                class: class.clone(),
                ..Default::default()
            }),
            sub: c.clone(),
            ..Default::default()
        });
        rsx! {
            Dynamic {
                brick: brick,
                {children}
            }
        }
    } else {
        rsx! {}
    }
}
