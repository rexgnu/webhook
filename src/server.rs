use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::request::CapturedRequest;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub tx: mpsc::UnboundedSender<CapturedRequest>,
}

pub async fn run_server(
    config: Config,
    tx: mpsc::UnboundedSender<CapturedRequest>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = AppState {
        config: Arc::new(config.clone()),
        tx,
    };

    let app = Router::new()
        .route("/{*path}", any(catch_all_handler))
        .route("/", any(catch_all_handler))
        .with_state(state);

    let addr = config.address();
    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn catch_all_handler(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    request: Request<Body>,
) -> impl IntoResponse {
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());

    // Extract headers into HashMap
    let headers_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Extract body
    let body_bytes: Bytes = request
        .into_body()
        .collect()
        .await
        .map(|b| b.to_bytes())
        .unwrap_or_default();

    let body = if body_bytes.is_empty() {
        None
    } else {
        String::from_utf8(body_bytes.to_vec()).ok()
    };

    // Create captured request
    let id = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let captured = CapturedRequest::new(
        id,
        method.to_string(),
        path.clone(),
        query,
        headers_map,
        body,
    );

    // Send to TUI (ignore error if receiver is dropped)
    let _ = state.tx.send(captured);

    // Get configured response
    let response_config = state.config.get_response(method.as_str(), &path);

    // Build response
    let status = StatusCode::from_u16(response_config.status).unwrap_or(StatusCode::OK);

    let mut response = Response::builder().status(status);

    for (key, value) in &response_config.headers {
        response = response.header(key, value);
    }

    response
        .body(Body::from(response_config.body.clone()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap()
        })
}
