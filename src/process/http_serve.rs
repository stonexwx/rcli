use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use tracing::{error, info};

#[derive(Debug)]
struct HttpServerState {
    path: PathBuf,
}

pub async fn process_http_server(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on port {}", addr, port);

    let state = HttpServerState { path };
    // axum router
    let router = axum::Router::new()
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));
    let listerner = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listerner, router).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServerState>>,
    Path(path): Path<String>,
) -> (StatusCode, String) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Requesting file: {:?}", p);
    if p.exists() {
        match tokio::fs::read(p).await {
            Ok(content) => {
                return (
                    StatusCode::OK,
                    String::from_utf8_lossy(&content).to_string(),
                )
            }
            Err(e) => {
                error!("Error reading file: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
            }
        }
    } else {
        error!("File not found: {:?}", p.display());
        return (
            StatusCode::NOT_FOUND,
            format!("File not found: {:?}", p.display()),
        );
    }
}
