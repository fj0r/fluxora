#![allow(unused)]
mod libs;
use anyhow::{Result, bail};
use axum::{Router, extract::Json, routing::get};
use libs::admin::data_router;
use libs::config::{Config, LogFormat, Logic};
use libs::error::HttpResult;
use libs::postgres::connx;
use libs::shared::Shared;
use message::queue::MessageQueue;
use serde_json::Value;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, fmt::layer, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt,
};

use libs::db::Model;
use libs::handler::{ChatMessage, Envelope, handler};
use libs::logic::*;
use message::time::Created;
use url::Url;
use urlencoding::encode;

async fn is_ready() -> HttpResult<Json<Value>> {
    Ok(axum::Json("ok".into()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::new()?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match &cfg.trace.format {
        LogFormat::compact => {
            registry().with(layer().compact()).with(filter).init();
        }
        LogFormat::json => {
            registry().with(layer().json()).with(filter).init();
        }
    };

    dbg!(&cfg);

    let base_url = Url::parse(&cfg.gateway.base_url)?;
    let hc = reqwest::Client::new();
    for (k, v) in &cfg.hooks {
        let r = hc.post(base_url.join(&encode(k))?).json(v).send().await;
        info!("init hook {} [{}]", k, &r?.status());
    }

    let client = connx(&cfg.database).await?;
    let shared = Shared::new(Model(client));

    let queue = cfg.queue;

    let (outgo_tx, income_rx) = if !queue.disable {
        queue
            .split::<ChatMessage<Created>, Envelope<Created>>()
            .await
    } else {
        (None, None)
    };

    let Some(income_rx) = income_rx else {
        bail!("income channel invalid");
    };
    let Some(outgo_tx) = outgo_tx else {
        bail!("outgo channel invalid");
    };

    info!("run as: Logic::{:?}", cfg.logic);
    match cfg.logic {
        Logic::Chat => {
            let _ = handler(outgo_tx, income_rx, shared.clone(), chat).await;
        }
        Logic::Crm => {
            let _ = handler(outgo_tx, income_rx, shared.clone(), crm).await;
        }
        Logic::Echo => {
            let _ = handler(outgo_tx, income_rx, shared.clone(), echo).await;
        }
    }

    let app = Router::new()
        .nest("/v1", data_router())
        .route("/is_ready", get(is_ready))
        .with_state(shared);

    let addr = "0.0.0.0:3003";
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
