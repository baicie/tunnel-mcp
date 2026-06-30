use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::AllowRootsReadPolicy;
use super::tools::{handle_request, MCP_TOOLS};
use crate::product::status::McpServerStatus;
use anyhow::anyhow;
use axum::{extract::State, routing::post, Json, Router};
use log::{error, info};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Clone)]
struct McpState {
    policy: Arc<AllowRootsReadPolicy>,
}

#[derive(Default)]
struct McpRuntimeState {
    shutdown: Option<oneshot::Sender<()>>,
    running: bool,
    port: Option<u16>,
    authorized_roots: Vec<PathBuf>,
    last_error: Option<String>,
}

#[derive(Default, Clone)]
pub struct McpServerManager {
    state: Arc<Mutex<McpRuntimeState>>,
}

impl McpServerManager {
    pub async fn start(&self, port: u16, roots: Vec<PathBuf>) -> anyhow::Result<McpServerStatus> {
        {
            let state = self.state.lock().unwrap();
            if state.running {
                if state.port == Some(port) && state.authorized_roots == roots {
                    return Ok(self.status_with_config(port, roots));
                }

                return Err(anyhow!(
                    "MCP server is already running; stop it before changing port or authorized roots"
                ));
            }
        }

        let policy = Arc::new(AllowRootsReadPolicy::new(roots.clone())?);
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();

        let listener = match TcpListener::bind(addr).await {
            Ok(value) => value,
            Err(err) => {
                let message = format!("failed to bind MCP server on {addr}: {err}");
                *self.state.lock().unwrap() = McpRuntimeState {
                    last_error: Some(message.clone()),
                    ..McpRuntimeState::default()
                };
                return Err(anyhow!(message));
            }
        };

        let app = Router::new()
            .route("/mcp", post(handle_http_mcp))
            .with_state(McpState { policy });

        let (tx, rx) = oneshot::channel::<()>();

        {
            let mut state = self.state.lock().unwrap();
            state.shutdown = Some(tx);
            state.running = true;
            state.port = Some(port);
            state.authorized_roots = roots.clone();
            state.last_error = None;
        }

        let manager = self.clone();

        tokio::spawn(async move {
            info!("MCP server listening on {addr}");

            let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                let _ = rx.await;
            });

            if let Err(err) = server.await {
                error!("MCP server failed: {err}");
                let mut state = manager.state.lock().unwrap();
                state.last_error = Some(err.to_string());
            }

            manager.state.lock().unwrap().running = false;
        });

        Ok(self.status_with_config(port, roots))
    }

    pub fn stop(&self, fallback_port: u16, fallback_roots: Vec<PathBuf>) -> McpServerStatus {
        let mut state = self.state.lock().unwrap();

        if let Some(tx) = state.shutdown.take() {
            let _ = tx.send(());
        }

        state.running = false;

        drop(state);
        self.status_with_config(fallback_port, fallback_roots)
    }

    pub fn status_with_config(
        &self,
        fallback_port: u16,
        fallback_roots: Vec<PathBuf>,
    ) -> McpServerStatus {
        let state = self.state.lock().unwrap();

        let port = state.port.unwrap_or(fallback_port);
        let roots = if state.running {
            state.authorized_roots.clone()
        } else {
            fallback_roots
        };

        let authorized_roots: Vec<String> = roots
            .into_iter()
            .map(|root| root.to_string_lossy().to_string())
            .collect();

        McpServerStatus {
            running: state.running,
            port,
            tools: MCP_TOOLS.iter().map(|value| value.to_string()).collect(),
            resources: authorized_roots
                .iter()
                .map(|root| format!("filesystem:{root}"))
                .collect(),
            authorized_roots,
            last_error: state.last_error.clone(),
        }
    }
}

async fn handle_http_mcp(
    State(state): State<McpState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    Json(handle_request(request, state.policy.as_ref()))
}
