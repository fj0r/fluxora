use super::super::store::Status;
use super::{Dynamic, Frame};
use crate::libs::hooks::{use_common_css, use_source_id};
use brick::classify::Classify;
use brick::{Brick, BrickOps, Rack, RackAttr};
use dioxus::{CapturedError, prelude::*};
use std::collections::hash_map::HashMap;

#[derive(Debug)]
struct ItemContainer {
    default: Option<Brick>,
    index: HashMap<String, Brick>,
}

impl From<Vec<Brick>> for ItemContainer {
    fn from(data: Vec<Brick>) -> Self {
        let mut default = None;
        let mut index = HashMap::new();
        for l in &data {
            if let Some(x) = l.get_selector() {
                index.insert(x.to_owned(), l.clone());
            } else {
                default = Some(l.clone());
            };
        }
        ItemContainer { index, default }
    }
}

impl ItemContainer {
    fn select(&self, child: &Brick) -> Option<Brick> {
        if let Some(s) = child.get_selector()
            && let Some(i) = self.index.get(s)
        {
            return Some(i).cloned();
        }
        self.default.clone()
    }
}

#[component]
pub fn rack_(id: Option<String>, brick: Rack, children: Element) -> Element {
    let mut css = vec!["rack", "f"];
    use_common_css(&mut css, &brick);

    let item: ItemContainer = brick.item.clone().context("item")?.into();
    let Some(source) = use_source_id(&brick) else {
        return Err(RenderError::Error(CapturedError::from_display("no event")));
    };

    let store = use_context::<Status>();
    let c = store.list.read();
    let c = c.get(source).cloned().unwrap_or_else(Vec::new);
    let r = c.iter().enumerate().map(|(idx, child)| {
        let key = child.get_id().clone().unwrap_or(idx.to_string());
        let brick = item.select(child);
        if let Some(brick) = brick {
            let x = rsx! {
                Frame {
                    brick: child.clone()
                }
            };
            if c.len() - 1 == idx {
                // last element
                rsx! {
                    Dynamic {
                        key: "{key}",
                        brick: brick,
                        {x}
                    }
                }
            } else {
                rsx! {
                    Dynamic {
                        key: "{key}",
                        brick: brick,
                        {x}
                    }
                }
            }
        } else {
            rsx! {
                Frame {
                    key: "{key}",
                    brick: child.clone()
                }
            }
        }
    });

    if let Some(RackAttr { scroll: x, .. }) = brick.attrs
        && x
    {
        let sl = store.list;
        if let Some(id) = &id {
            let id = id.clone();
            use_effect(move || {
                // TODO: fine-grained
                let _ = sl.read();
                document::eval(&format! {
                    r#"
                    var e = document.getElementById("{id}");
                    if (Math.abs(e.scrollHeight - e.offsetHeight - e.scrollTop) < e.offsetHeight) {{
                        e.scrollTop = e.scrollHeight;
                    }}
                    "#
                });
            });
        }
    };

    rsx! {
        div {
            id: id,
            class: css.join(" "),
            {r}
        }
    }
}
