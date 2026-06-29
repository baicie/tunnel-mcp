use crate::product::settings::TunnelSettings;
use crate::product::status::{initial_tunnel_status, TunnelHealthState, TunnelStatus};
use crate::product::tunnel::client_health::{is_local_port_open, local_mcp_url};
use anyhow::{anyhow, Context};
use log::{debug, error, warn};
use serde::Serialize;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_LOG_LINES: usize = 500;
const OPENAI_KEY_ENV: &str = "TUNNEL_MCP_OPENAI_API_KEY";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TunnelClientLogLine {
    pub stream: String,
    pub line: String,
}

#[derive(Default, Clone)]
pub struct TunnelProcessManager {
    child: Arc<Mutex<Option<Child>>>,
    last_error: Arc<Mutex<Option<String>>>,
    logs: Arc<Mutex<VecDeque<TunnelClientLogLine>>>,
}

impl TunnelProcessManager {
    pub fn start(&self, settings: &TunnelSettings) -> anyhow::Result<TunnelStatus> {
        self.reap_exited_child();

        if self.is_running() {
            return self.status(settings);
        }

        let binary = settings
            .tunnel_client_path
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("tunnel-client is not installed"))?;

        let tunnel_id = settings
            .tunnel_id
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("tunnel id is not configured"))?;

        let openai_key = settings
            .openai_api_key
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow!("OpenAI key is not configured"))?;

        let endpoint = local_mcp_url(settings.mcp_server_port);

        let mut command = Command::new(binary);
        command
            .arg("--tunnel-id")
            .arg(tunnel_id)
            .arg("--openai-key-env")
            .arg(OPENAI_KEY_ENV)
            .arg("--local-mcp-url")
            .arg(&endpoint)
            .env(OPENAI_KEY_ENV, openai_key)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to start tunnel-client at {}", binary))?;

        if let Some(stdout) = child.stdout.take() {
            spawn_log_thread("stdout", BufReader::new(stdout), Arc::clone(&self.logs));
        }

        if let Some(stderr) = child.stderr.take() {
            spawn_log_thread("stderr", BufReader::new(stderr), Arc::clone(&self.logs));
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

    pub fn restart(&self, settings: &TunnelSettings) -> anyhow::Result<TunnelStatus> {
        let _ = self.stop(settings);
        self.start(settings)
    }

    pub fn status(&self, settings: &TunnelSettings) -> anyhow::Result<TunnelStatus> {
        self.reap_exited_child();

        let mut running = false;
        let mut pid = None;

        if let Some(child) = self.child.lock().unwrap().as_mut() {
            running = true;
            pid = Some(child.id());
        }

        let local_mcp_port_open = is_local_port_open(settings.mcp_server_port);
        let mut last_error = self.last_error.lock().unwrap().clone();

        let health = if running && local_mcp_port_open {
            TunnelHealthState::Healthy
        } else if running && !local_mcp_port_open {
            if last_error.is_none() {
                last_error = Some(format!(
                    "local MCP endpoint {} is not reachable yet",
                    local_mcp_url(settings.mcp_server_port)
                ));
            }
            TunnelHealthState::Warning
        } else if last_error.is_some() {
            TunnelHealthState::Unhealthy
        } else {
            TunnelHealthState::Unknown
        };

        let mut status = initial_tunnel_status(
            settings.tunnel_client_path.clone(),
            settings.tunnel_client_version.clone(),
            local_mcp_port_open,
        );

        status.running = running;
        status.pid = pid;
        status.endpoint = Some(local_mcp_url(settings.mcp_server_port));
        status.health = health;
        status.last_error = last_error;

        Ok(status)
    }

    pub fn logs(&self) -> Vec<TunnelClientLogLine> {
        self.logs.lock().unwrap().iter().cloned().collect()
    }

    pub fn is_running(&self) -> bool {
        self.reap_exited_child();
        self.child.lock().unwrap().is_some()
    }

    fn reap_exited_child(&self) {
        let mut guard = self.child.lock().unwrap();

        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let message = format!("tunnel-client exited with {status}");
                    warn!("{message}");
                    *self.last_error.lock().unwrap() = Some(message);
                    *guard = None;
                }
                Ok(None) => {}
                Err(err) => {
                    error!("tunnel-client wait failed: {err}");
                    *self.last_error.lock().unwrap() = Some(err.to_string());
                    *guard = None;
                }
            }
        }
    }
}

fn spawn_log_thread<R: BufRead + Send + 'static>(
    stream: &'static str,
    reader: R,
    logs: Arc<Mutex<VecDeque<TunnelClientLogLine>>>,
) {
    thread::Builder::new()
        .name(format!("tunnel-client-log-{stream}"))
        .spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        debug!("[tunnel-client.{stream}] {line}");
                        push_log_line(&logs, stream, line);
                    }
                    Err(err) => {
                        warn!("[tunnel-client.{stream}] read error: {err}");
                        push_log_line(&logs, stream, format!("read error: {err}"));
                        break;
                    }
                }
            }
        })
        .expect("spawn tunnel-client log thread");
}

fn push_log_line(logs: &Arc<Mutex<VecDeque<TunnelClientLogLine>>>, stream: &str, line: String) {
    let mut guard = logs.lock().unwrap();

    if guard.len() >= MAX_LOG_LINES {
        guard.pop_front();
    }

    guard.push_back(TunnelClientLogLine {
        stream: stream.to_string(),
        line,
    });
}
