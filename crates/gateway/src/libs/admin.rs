use super::config::{ASSETS_PATH, Config, Hooks};
use super::error::HttpResult;
use super::shared::{Arw, Arwsc, Sender, StateChat};
use axum::{
    Router,
    extract::{Json, Path, Request, State},
    http::{StatusCode, header::ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use indexmap::IndexMap;
use message::time::Created;
use message::{
    Envelope,
    session::{Session, SessionCount, SessionInfo},
};
use minijinja::Environment;
use serde_json::{Map, Value, from_str};

async fn send(
    State(session): State<Arwsc<Sender>>,
    Json(payload): Json<Envelope<Created>>,
) -> HttpResult<(StatusCode, Json<Vec<Session>>)> {
    let mut succ: Vec<Session> = Vec::new();
    let s = session.read().await;
    if payload.receiver.is_empty() {
        for (n, c) in &*s {
            let _ = c.send(payload.message.clone());
            succ.push(n.to_owned());
        }
    } else {
        for r in payload.receiver {
            if s.contains_key(&r)
                && let Some(x) = s.get(&r)
            {
                let _ = x.send(payload.message.clone());
                succ.push(r);
            }
        }
    }
    Ok((StatusCode::OK, succ.into()))
}

async fn list(State(session): State<Arwsc<Sender>>) -> axum::Json<Vec<Session>> {
    let s = session.read().await;
    let mut r = Vec::new();
    for (k, _v) in &*s {
        r.push(k.clone());
    }
    Json(r)
}

async fn info(
    Path(user): Path<String>,
    State(session): State<Arwsc<Sender>>,
) -> axum::Json<Map<String, Value>> {
    let s = session.read().await;
    let u = s.get(&user.as_str().into()).map(|x| x.info.clone());
    Json(u.unwrap_or_else(Map::new))
}

struct Req<'a>(&'a Request);
impl std::fmt::Display for Req<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "### {} {}", self.0.method(), self.0.uri());
        for (name, value) in self.0.headers() {
            let _ = writeln!(f, "  | {}: {:?}", name, value);
        }
        Ok(())
    }
}

pub fn admin_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/sessions", get(list))
        .route("/info/{user}", get(info))
        .route("/send", post(send))
}

async fn render(Path(name): Path<String>, Json(payload): Json<Value>) -> HttpResult<Response> {
    let mut env = Environment::new();
    let path = std::path::Path::new(ASSETS_PATH);
    let content = async_fs::read_to_string(path.join(&name)).await?;
    let _ = env.add_template_owned(&name, content);
    let r = env.get_template(&name)?.render(payload)?;
    Ok(Response::new(r.into()))
}

async fn echo(req: Request) -> HttpResult<Response> {
    println!("{}", Req(&req));
    match req.headers().get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"application/json") => {
            let body = req.into_body();
            let limit = 204800usize;
            let by = axum::body::to_bytes(body, limit).await?;
            let s = String::from_utf8(by.to_vec())?;
            Ok(Json(from_str::<Value>(&s)?).into_response())
        }
        _ => Ok(req.into_body().into_response()),
    }
}

async fn login(
    State(_state): State<StateChat<Sender>>,
    Json(mut payload): Json<Map<String, Value>>,
) -> HttpResult<Json<SessionInfo>> {
    use short_uuid::ShortUuid;
    let uuid = ShortUuid::generate().to_string();
    payload.insert("username".into(), uuid[..6].into());
    Ok(Json(SessionInfo {
        id: uuid.as_str().into(),
        info: payload,
    }))
}

async fn logout(
    State(_state): State<StateChat<Sender>>,
    Json(payload): Json<Map<String, Value>>,
) -> HttpResult<Json<SessionInfo>> {
    Ok(Json(SessionInfo {
        id: "".into(),
        info: payload,
    }))
}

async fn inc(
    State(count): State<Arw<SessionCount>>,
    Json(payload): Json<Map<String, Value>>,
) -> HttpResult<String> {
    let mut count = count.write().await;
    *count += 1;
    let c = *count;
    drop(count);
    if let Some(interval) = payload.get("interval").and_then(|x| x.as_u64()) {
        use tokio::time::{Duration, sleep};
        let _ = sleep(Duration::from_secs(interval)).await;
    };
    Ok(c.to_string())
}

async fn health(State(state): State<StateChat<Sender>>) -> HttpResult<Json<Value>> {
    let mut b = Map::new();
    let count = state.count.read().await;
    b.insert("count".to_string(), (*count as u64).into());
    Ok(axum::Json(Value::Object(b)))
}

pub fn debug_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/render/{name}", post(render))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/echo", post(echo))
        .route("/inc", post(inc))
        .route("/health", get(health))
}

async fn list_hook(
    State(config): State<Arw<Config>>,
) -> HttpResult<(StatusCode, Json<IndexMap<String, Hooks>>)> {
    let s = config.read().await.clone();
    Ok((StatusCode::OK, Json(s.hooks)))
}

async fn update_hook(
    Path(hook): Path<String>,
    State(config): State<Arw<Config>>,
    Json(payload): Json<Hooks>,
) -> HttpResult<(StatusCode, Json<bool>)> {
    let mut s = config.write().await;
    s.hooks.insert(hook, payload);
    Ok((StatusCode::OK, Json(true)))
}

pub fn config_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/hooks", get(list_hook))
        .route("/hooks/{hook}", post(update_hook))
}
