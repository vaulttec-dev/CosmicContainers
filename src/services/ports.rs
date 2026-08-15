//! Host listening ports (`ss`) and the processes holding them (`kill`).

use std::time::Duration;

use crate::{PortEntry, parse_ss_output};

use super::docker::{run_host, stop_container};

/// A signal is asynchronous; wait this long before re-reading the socket list.
const KILL_SETTLE: Duration = Duration::from_millis(400);

/// List listening TCP/UDP sockets (`ss -tulpnH`).
///
/// Sockets owned by other users (root, so every `docker-proxy`) are listed without a PID:
/// `ss` only reveals process details for the current user.
pub async fn list_ports() -> Result<Vec<PortEntry>, String> {
    let output = run_host("ss", &["-tulpnH"]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_ss_failure(&stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ss_output(&stdout))
}

/// Signal a process by PID (`kill -TERM`, or `-KILL` when `force`).
pub async fn kill_process(pid: u32, force: bool) -> Result<(), String> {
    let signal = if force { "-KILL" } else { "-TERM" };
    let pid = pid.to_string();
    let output = run_host("kill", &[signal, &pid]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_kill_failure(&stderr))
}

pub async fn kill_then_list(pid: u32, force: bool) -> Result<Vec<PortEntry>, String> {
    kill_process(pid, force).await?;
    tokio::time::sleep(KILL_SETTLE).await;
    list_ports().await
}

/// Free a published port by stopping the container that owns it.
pub async fn stop_container_then_list(name: String) -> Result<Vec<PortEntry>, String> {
    stop_container(&name).await?;
    list_ports().await
}

fn format_ss_failure(stderr: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return "ss failed. Install iproute2 to list listening ports.".to_string();
    }
    stderr.to_string()
}

fn format_kill_failure(stderr: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return "kill failed".to_string();
    }
    if stderr.contains("not permitted") {
        return format!("{stderr}\n\nHint: this process belongs to another user.");
    }
    stderr.to_string()
}
