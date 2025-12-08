use crate::libs::components::Frame;
use crate::libs::hooks::use_common_css;
use crate::libs::store::Status;
use brick::{Bind, BindVariant, BrickOps, Case, CaseAttr, Placeholder};
use dioxus::prelude::*;

#[component]
pub fn case_(id: Option<String>, brick: Case, children: Element) -> Element {
    let mut css = vec!["case", "f"];
    if let Some(id) = &id {
        css.push(id);
    }
    let mut style = String::new();
    let Case { attrs, .. } = &brick;

    let mut f = true;
    if let Some(CaseAttr { grid, .. }) = attrs {
        if let Some(g) = grid {
            f = false;
            css.push("g");
            style = g
                .iter()
                .map(|(k, v)| format!("{}: {};", k, v.as_str().unwrap()))
                .collect::<Vec<String>>()
                .join("\n");
        };
        if f {
            css.push("f");
        };
    };
    use_common_css(&mut css, &brick);

    rsx! {
        div {
            class: css.join(" "),
            style: style,
            {children}
        }
    }
}

#[component]
pub fn placeholder_(id: Option<String>, brick: Placeholder, children: Element) -> Element {
    let mut css = vec!["placeholder", "f"];
    use_common_css(&mut css, &brick);
    let store = use_context::<Status>();
    let s = store.data.read();

    if let Some(x) = brick.get_bind()
        && let Some(Bind {
            variant: BindVariant::Source { source },
            default: _,
            r#type: _kind,
        }) = x.get("value")
        && let Some(data) = s.get(source)
        && let Some(eid) = id.clone()
    {
        use_effect(move || {
            let js = format!(
                r#"
                let x = document.getElementById('{eid}');
                x.classList.add('fade-in-and-out');
                setTimeout(() => x.classList.remove('fade-in-and-out'), 1000);
                "#
            );
            document::eval(&js);
        });
        rsx! {
            div {
                id: id,
                class: css.join(" "),
                Frame { brick: data.clone() }
            }
        }
    } else {
        rsx! {
            div {
                id: id,
                class: css.join(" "),
                {children}
            }
        }
    }
}
