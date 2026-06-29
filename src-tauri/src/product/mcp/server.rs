use super::protocol::{JsonRpcRequest, JsonRpcResponse};
use super::resources::AllowRootsReadPolicy;
use super::tools::{handle_request, MCP_RESOURCES, MCP_TOOLS};
use crate::product::status::McpServerStatus;
use axum::{extract::State, routing::post, Json, Router};
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
pub struct McpServerManager {
    shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    running: Arc<Mutex<bool>>,
    port: u16,
}

impl McpServerManager {
    pub fn new(port: u16) -> Self {
        Self {
            shutdown: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            port,
        }
    }

    pub async fn start(&self, roots: Vec<PathBuf>) -> anyhow::Result<McpServerStatus> {
        if *self.running.lock().unwrap() {
            return Ok(self.status());
        }

        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        let listener = TcpListener::bind(addr).await?;
        let policy = Arc::new(AllowRootsReadPolicy::new(roots)?);
        let state = McpState { policy };
        let app = Router::new()
            .route("/mcp", post(handle_http_mcp))
            .with_state(state);
        let (tx, rx) = oneshot::channel::<()>();
        *self.shutdown.lock().unwrap() = Some(tx);
        *self.running.lock().unwrap() = true;

        let running = self.running.clone();
        tokio::spawn(async move {
            let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                let _ = rx.await;
            });
            let _ = server.await;
            *running.lock().unwrap() = false;
        });

        Ok(self.status())
    }

    pub fn stop(&self) -> McpServerStatus {
        if let Some(tx) = self.shutdown.lock().unwrap().take() {
            let _ = tx.send(());
        }
        *self.running.lock().unwrap() = false;
        self.status()
    }

    pub fn status(&self) -> McpServerStatus {
        McpServerStatus {
            running: *self.running.lock().unwrap(),
            port: self.port,
            tools: MCP_TOOLS.iter().map(|value| value.to_string()).collect(),
            resources: MCP_RESOURCES
                .iter()
                .map(|value| value.to_string())
                .collect(),
        }
    }
}

async fn handle_http_mcp(
    State(state): State<McpState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    Json(handle_request(request, state.policy.as_ref()))
}
