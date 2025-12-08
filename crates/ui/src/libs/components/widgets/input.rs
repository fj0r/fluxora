use super::super::super::store::Status;
use crate::libs::hooks::use_common_css;
use brick::{Bind, BindVariant, Brick, BrickOps, Input, JsType, Text};
use dioxus::prelude::*;
use maplit::hashmap;
use serde_json::{Value, to_value};
use std::ops::Deref;
use std::rc::Rc;

fn default_option_jskind(v: &Option<JsType>) -> Value {
    v.as_ref()
        .map(|x| x.default_value())
        .unwrap_or_else(|| to_value("").unwrap())
}

#[component]
pub fn input_(id: Option<String>, brick: Input) -> Element {
    let store = use_context::<Status>();
    let mut css = vec!["input", "f", "shadow"];
    use_common_css(&mut css, &brick);

    let (bind_type, key, kind, signal) = brick
        .get_bind()
        .and_then(|x| x.get("value"))
        .cloned()
        .map(|x| match x {
            Bind {
                variant: BindVariant::Field { field, signal, .. },
                r#type: kind,
                ..
            } => ("field", field, kind, signal),
            Bind {
                variant: BindVariant::Event { event },
                r#type: kind,
                ..
            } => ("event", event, kind, None),
            _ => ("", "".to_string(), Default::default(), None),
        })
        .unwrap();

    let mut slot = signal.unwrap_or_else(|| use_signal(|| default_option_jskind(&kind)));
    let key = Rc::new(key);
    let kind = Rc::new(kind);

    let k1 = kind.clone();
    let k3 = key.clone();
    let mut s1 = store.clone();
    let oninput = move |event: Event<FormData>| {
        let event_value = event.value();
        let parsed_value = match *k1 {
            Some(JsType::bool) => to_value(event_value == "true"),
            Some(JsType::number) => to_value(event_value.parse::<f64>().unwrap()),
            _ => to_value(event_value),
        }
        .unwrap();
        match bind_type {
            "field" => slot.set(parsed_value),
            "variable" => {
                s1.set(
                    k3.deref(),
                    Brick::text(Text {
                        bind: Some(hashmap! {
                            "value".to_owned() => Bind { default: Some(parsed_value), ..Default::default() }
                        }),
                        ..Default::default()
                    })
                    ,
                );
            }
            _ => slot.set(parsed_value),
        };
    };

    let k2 = kind.clone();
    let onkeydown = move |ev: Event<KeyboardData>| {
        let mut s = store.clone();
        let event = key.clone();
        let kind = k2.clone();
        let val = slot();
        async move {
            if ev.data.key() == Key::Enter {
                match bind_type {
                    "field" => {
                        if let Some(mut sig) = signal {
                            sig.set(val);
                        };
                    }
                    "event" => {
                        s.send(event.deref(), None, val).await;
                        *slot.write() = default_option_jskind(&kind)
                    }
                    _ => {}
                }
            }
        }
    };

    match *kind {
        Some(JsType::number) => {
            let v = slot.read().as_f64();
            rsx! {
                input {
                    class: css.join(" "),
                    type: JsType::input_type(&JsType::number),
                    value: v,
                    oninput: oninput,
                    onkeydown: onkeydown
                }
            }
        }
        Some(JsType::bool) => {
            let v = slot.read().as_bool();
            rsx! {
                input {
                    class: css.join(" "),
                    type: JsType::input_type(&JsType::bool),
                    value: v,
                    oninput: oninput,
                    onkeydown: onkeydown
                }
            }
        }
        Some(ref x) => {
            let v = slot.read().as_str().unwrap_or("").to_string();
            rsx! {
                input {
                    class: css.join(" "),
                    type: JsType::input_type(x),
                    value: v,
                    oninput: oninput,
                    onkeydown: onkeydown
                }
            }
        }
        _ => {
            let v = slot.read().as_str().unwrap_or("").to_string();
            rsx! {
                input {
                    class: css.join(" "),
                    type: JsType::input_type(&JsType::text),
                    value: v,
                    oninput: oninput,
                    onkeydown: onkeydown
                }
            }
        }
    }
}
