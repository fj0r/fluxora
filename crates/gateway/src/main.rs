mod libs;
use anyhow::{Ok as Okk, Result, bail};
use arc_swap::ArcSwap;
use axum::{
    Router,
    extract::{Query, State, ws::WebSocketUpgrade},
    http::{HeaderMap, Response, StatusCode},
    routing::get,
};
use axum_extra::extract::cookie::CookieJar;
use libs::admin::*;
use libs::config::{ASSETS_PATH, Config, LiveConfig, LogFormat};
use libs::shared::{Sender, StateChat};
use libs::template::Tmpls;
use libs::websocket::{handle_ws, send_to_ws};
use message::queue::MessageQueue;
use serde_json::{Map, Value};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, fmt::layer, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
    #[allow(unused_mut)]
    let mut config = LiveConfig::new()?;
    // config.listen().await.unwrap();
    dbg!(&config.data);

    let config = Config::new()?;
    // console_subscriber::init();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match &config.trace.format {
        LogFormat::compact => {
            registry().with(layer().compact()).with(filter).init();
        }
        LogFormat::json => {
            registry().with(layer().json()).with(filter).init();
        }
    };

    let config = Arc::new(ArcSwap::from_pointee(config));
    //dbg!(&config);
    let tmpls: Arc<Tmpls<'static>> = Arc::new(Tmpls::new(ASSETS_PATH).unwrap());

    let shared = StateChat::<Sender>::new(config.clone());

    let queue = config.load().queue.clone();

    let (outgo_tx, income_rx) = if !queue.disable {
        queue.split().await
    } else {
        (None, None)
    };

    let Some(rx) = income_rx else {
        bail!("income channel invalid");
    };
    let Some(tx) = outgo_tx else {
        bail!("outgo channel invalid");
    };

    send_to_ws(rx, &shared).await;

    let app = Router::new()
        .route(
            "/channel",
            get(
                |ws: WebSocketUpgrade,
                 Query(mut q): Query<Map<String, Value>>,
                 jar: CookieJar,
                 State(state): State<StateChat<Sender>>| async move {
                    let s = state.config.load();
                    let login_with_cookie = s.login_with_cookie;
                    let login = &s.hooks.get("login").unwrap()[0];

                    if login_with_cookie {
                        let cookie: Value = jar.iter().map(|c| (c.name(), c.value())).collect();
                        q.insert("Cookie".to_owned(), cookie);
                    }

                    let Ok(a) = login.handle(&q, tmpls.clone()).await else {
                        return Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body("UNAUTHORIZED".into())
                            .unwrap();
                    };

                    let logout = s.hooks.get("logout").unwrap()[0].clone();
                    drop(s);
                    ws.on_upgrade(async move |socket| {
                        handle_ws(socket, tx, state, config, tmpls.clone(), &a).await;
                        let _ = logout.handle::<Value>(&a.into(), tmpls.clone()).await;
                    })
                },
            ),
        )
        .nest("/admin", admin_router())
        .nest("/config", config_router())
        .nest("/debug", debug_router())
        .fallback_service(ServeDir::new("./static"))
        .with_state(shared);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Okk(())
}
