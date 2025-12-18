use super::chart::chart_;
use super::container::*;
use super::diagram::diagram_;
use super::float::float_;
use super::fold::fold_;
use super::form::form_;
use super::popup::popup_;
use super::rack::rack_;
use super::render::render_;
use super::svg::*;
use super::widgets::*;
use brick::Brick;
use dioxus::prelude::*;
use ui_macro::gen_dispatch;

use std::sync::{LazyLock, Mutex};
static COMPONENT_ID: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

#[component]
pub fn Dynamic(brick: Brick, children: Element) -> Element {
    let id = if cfg!(debug_assertions) {
        let mut tc = COMPONENT_ID.lock().unwrap();
        *tc += 1;
        Some(format!("={}=", *tc))
    } else {
        None
    };

    let c = {
        gen_dispatch! {
            file = "../brick/src/lib.rs",
            entry = "Brick",
            object = "brick"
        }
    };
    rsx! {
        {c}
    }
}
