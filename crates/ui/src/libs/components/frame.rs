use super::Dynamic;
use brick::{Brick, BrickOps};
use dioxus::prelude::*;

#[component]
pub fn Frame(brick: Brick) -> Element {
    let sub = brick.borrow_sub();
    if let Some(sub) = sub {
        let sub = sub.iter().map(|c| {
            rsx! {
                Frame { brick: c.clone() }
            }
        });

        rsx! {
            Dynamic {
                brick: brick.clone(),
                {sub}
            }
        }
    } else {
        rsx! {
            Dynamic { brick }
        }
    }
}
