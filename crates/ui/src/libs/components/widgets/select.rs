use crate::libs::components::Frame;
use crate::libs::hooks::{use_common_css, use_source_list, use_source_value, use_target_value};
use brick::{BrickOps, Select, classify::Classify};
use dioxus::prelude::*;
use serde_json::{Value, to_value};
use std::rc::Rc;

#[component]
pub fn select_(id: Option<String>, brick: Select, children: Element) -> Element {
    let mut css = vec!["select", "f"];
    let brick = Rc::new(brick);
    use_common_css(&mut css, &*brick);
    let option = use_source_list(&*brick, "options");
    let current = use_source_value(&*brick);
    let mut current = use_signal(|| {
        current
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or("".to_string())
    });
    let mkclick = |value: Value| {
        let brick = brick.clone();
        move |_: MouseEvent| {
            let emitter = use_target_value(&*brick);
            emitter.map(|x| x(value.clone()));
            if let Some(v) = value.as_str() {
                current.set(v.to_string());
            }
        }
    };
    if let Some(option) = option {
        let children = option.iter().enumerate().map(|(idx, child)| {
            let key = child.get_id().clone().unwrap_or(idx.to_string());
            let current = current();
            let mut child = child.clone();
            child.add_class("f as-stretch");
            if current == key {
                child.add_class("selected");
                rsx! {
                    div {
                        Frame {
                            key: "{key}",
                            brick: child
                        }
                    }
                }
            } else {
                let v = match to_value(&key) {
                    Ok(v) => v,
                    Err(_) => to_value("").unwrap(),
                };
                rsx! {
                    div {
                        onclick: mkclick(v),
                        Frame {
                            key: "{key}",
                            brick: child
                        }
                    }
                }
            }
        });
        rsx! {
            div {
                class: css.join(" "),
                {children}
            }
        }
    } else {
        rsx!()
    }
}
