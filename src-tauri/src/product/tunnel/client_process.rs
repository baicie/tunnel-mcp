use crate::product::settings::TunnelSettings;
use crate::product::status::{initial_tunnel_status, TunnelStatus};
use anyhow::{anyhow, Context};
use log::{debug, error, warn};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Manages the local `tunnel-client` child process. A single instance
/// is registered as a Tauri-managed state via
/// `lib.rs::run`; all command handlers share it through `State<...>`.
#[derive(Default, Clone)]
pub struct TunnelProcessManager {
    child: Arc<Mutex<Option<Child>>>,
    last_error: Arc<Mutex<Option<String>>>,
}

impl TunnelProcessManager {
    pub fn start(
        &self,
        settings: &TunnelSettings,
        local_mcp_url: &str,
    ) -> anyhow::Result<TunnelStatus> {
        if self.is_running() {
            return self.status(settings);
        }

        let binary = settings
            .tunnel_client_path
            .as_ref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("tunnel-client is not installed"))?;

        let tunnel_id = settings
            .tunnel_id
            .as_ref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("tunnel id is not configured"))?;

        let openai_key = settings
            .openai_api_key
            .as_ref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("OpenAI key is not configured"))?;

        let mut command = Command::new(binary);
        command
            .arg("--tunnel-id")
            .arg(tunnel_id)
            .arg("--openai-key")
            .arg(openai_key)
            .arg("--local-mcp-url")
            .arg(local_mcp_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to start tunnel-client at {}", binary))?;

        if let Some(stdout) = child.stdout.take() {
            spawn_log_thread("tunnel-client.stdout", BufReader::new(stdout));
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_log_thread("tunnel-client.stderr", BufReader::new(stderr));
        }

        *self.child.lock().unwrap() = Some(child);
        *self.last_error.lock().unwrap() = None;
        self.status(settings)
    }

    pub fn stop(&self, settings: &TunnelSettings) -> anyhow::Result<TunnelStatus> {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.status(settings)
    }

    pub fn restart(
        &self,
        settings: &TunnelSettings,
        local_mcp_url: &str,
    ) -> anyhow::Result<TunnelStatus> {
        let _ = self.stop(settings);
        self.start(settings, local_mcp_url)
    }

    pub fn status(&self, settings: &TunnelSettings) -> anyhow::Result<TunnelStatus> {
        let mut guard = self.child.lock().unwrap();
        let mut running = false;
        let mut pid = None;

        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let message = format!("tunnel-client exited with {}", status);
                    warn!("{message}");
                    *self.last_error.lock().unwrap() = Some(message);
                    *guard = None;
                }
                Ok(None) => {
                    running = true;
                    pid = Some(child.id());
                }
                Err(err) => {
                    error!("tunnel-client wait failed: {err}");
                    *self.last_error.lock().unwrap() = Some(err.to_string());
                    *guard = None;
                }
            }
        }

        let mut status = initial_tunnel_status(settings.tunnel_client_path.clone());
        status.running = running;
        status.pid = pid;
        status.last_error = self.last_error.lock().unwrap().clone();
        Ok(status)
    }

    pub fn is_running(&self) -> bool {
        self.child
            .lock()
            .unwrap()
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_none())
    }
}

fn spawn_log_thread<R: BufRead + Send + 'static>(prefix: &'static str, reader: R) {
    thread::Builder::new()
        .name(format!("tunnel-client-log-{prefix}"))
        .spawn(move || {
            let mut stream = reader.lines();
            loop {
                match stream.next() {
                    Some(Ok(line)) => debug!("[{prefix}] {line}"),
                    Some(Err(err)) => {
                        warn!("[{prefix}] read error: {err}");
                        break;
                    }
                    None => break,
                }
            }
        })
        .expect("spawn tunnel-client log thread");
}
