use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use tracing::{error, info};

use tower_http::services::ServeDir;

#[derive(Debug)]
struct HttpServerState {
    path: PathBuf,
}

pub async fn process_http_server(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on port {}", addr, port);

    let state = HttpServerState { path };
    let serve_dir = ServeDir::new(state.path.clone())
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br()
        .precompressed_zstd();
    // axum router
    let router = axum::Router::new()
        .route("/*path", get(file_handler))
        .nest_service("/tower", serve_dir)
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
            Ok(content) => (
                StatusCode::OK,
                String::from_utf8_lossy(&content).to_string(),
            ),
            Err(e) => {
                error!("Error reading file: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }
    } else {
        error!("File not found: {:?}", p.display());
        (
            StatusCode::NOT_FOUND,
            format!("File not found: {:?}", p.display()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use axum::{
        extract::{Path, State},
        http::StatusCode,
    };

    use crate::process::http_serve::{file_handler, HttpServerState};

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServerState {
            path: PathBuf::from("."),
        });
        let (status, connect) =
            file_handler(State(state.clone()), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        assert!(connect.trim().starts_with("[package]"));
    }
}
