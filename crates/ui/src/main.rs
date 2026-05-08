mod libs;
use dioxus::prelude::*;
use libs::components::*;
use libs::store::{Status, use_status};
use message::codec::{ActiveCodec, CodecType};
use tracing_wasm::WASMLayerConfigBuilder;

#[allow(unused_macros)]
macro_rules! info {
    ($x: tt) => {
        dioxus::logger::tracing::info!("{} = {:#?}", stringify!($x), $x)
    };
}

static STATUS: GlobalSignal<Status> = Global::new(|| {
    let doc = web_sys::window().unwrap().document().unwrap();
    let loc = doc.location().unwrap();
    let mut host = "".to_owned();
    let mut token = None;
    let mut codec_type = CodecType::Cbor; // Default to Cbor
    if let Ok(Some(ele)) = doc.query_selector("#main") {
        if let Some(h) = ele.get_attribute("data-host") {
            host = h;
        } else {
            host = loc.host().unwrap();
        };

        if let Ok(href) = loc.href()
            && let Ok(href) = web_sys::Url::new(&href)
            && let Some(t) = href.search_params().get("token")
        {
            token = Some(t);
            // Extract codec from URL params if available, otherwise fallback to default
            if let Some(codec_str) = href.search_params().get("codec") {
                if let Ok(t) = codec_str.parse::<CodecType>() {
                    codec_type = t;
                }
            }
        } else if let Some(t) = ele.get_attribute("data-token") {
            token = Some(t);
        };
    };
    let query = if let Some(token) = token {
        format!("?token={}", &token)
    } else {
        "".to_owned()
    };
    let url = format!("ws://{}/channel{}", host, query);
    use_status(&url, ActiveCodec::new(codec_type)).expect("connecting failed")
});

fn main() {
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::new()
            .set_max_level(dioxus::logger::tracing::Level::INFO)
            .build(),
    );
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| STATUS());
    let layout = STATUS().layout;

    rsx! {
        document::Style { href: asset!("/assets/main.css") }
        document::Style { href: asset!("/assets/custom.css") }
        // document::Script { src: asset!("/assets/apexcharts.min.js") }
        // document::Script { src: asset!("/assets/mermaid.min.js") }
        Frame {
            brick: layout()
        }
    }
}
