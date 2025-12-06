use crate::libs::hooks::use_default;
use brick::{Bind, BindVariant, BrickOps, Button, ButtonAttr};
use dioxus::prelude::*;
use serde_json::{Value, to_value};

#[component]
pub fn button_(id: Option<String>, brick: Button) -> Element {
    let t = use_default(&brick)
        .unwrap_or(to_value("Ok").unwrap())
        .as_str()
        .unwrap()
        .to_owned();

    let oneshot = brick
        .attrs
        .as_ref()
        .map(|ButtonAttr { oneshot, .. }| *oneshot)
        .unwrap_or(false);

    if let Some(Bind {
        variant: BindVariant::Submit {
            signal: Some(mut s),
            ..
        },
        ..
    }) = brick.get_bind().and_then(|x| x.get("value").cloned())
    {
        let v = s.read().as_bool().unwrap();
        let mut css = vec!["button", "shadow"];
        css.push(if !v { "accent" } else { "disabled" });
        rsx! {
            button {
                class: css.join(" "),
                onclick: move |_event| {
                    if oneshot {
                        if !v {
                            s.set(Value::Bool(true));
                        }
                    } else {
                        s.set(Value::Bool(!v));
                        spawn(async move {
                            s.set(Value::Bool(v));
                        });
                    }
                },
                {t}
            }
        }
    } else {
        rsx! {}
    }
}
