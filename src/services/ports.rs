//! Host listening ports (`ss`) and the processes holding them (`kill`).

use std::time::Duration;

use crate::{PortEntry, SignalTarget, parse_ps_pgid_output, parse_ss_output, retain_dev_servers};

use super::docker::run_host;

/// A signal is asynchronous; wait this long before re-reading the socket list.
const KILL_SETTLE: Duration = Duration::from_millis(400);

/// List this user's listening TCP sockets.
///
/// `-u` stays in the flags even though UDP rows are dropped right after: without it `ss`
/// omits the `Netid` column entirely and every field shifts one position left, which the
/// parser reads as garbage. Filtering in code keeps the output shape constant.
pub async fn list_ports() -> Result<Vec<PortEntry>, String> {
    let output = run_host("ss", &["-tulpnH"]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_ss_failure(&stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = parse_ss_output(&stdout);
    retain_dev_servers(&mut entries);
    fill_process_groups(&mut entries).await;
    Ok(entries)
}

/// Read each listener's process group (`ss` does not report it) in one `ps` call.
///
/// Best effort: without a group the row falls back to signalling the bare PID.
async fn fill_process_groups(entries: &mut [PortEntry]) {
    let mut pids: Vec<String> = entries
        .iter()
        .filter_map(|entry| entry.pid.map(|pid| pid.to_string()))
        .collect();
    pids.sort_unstable();
    pids.dedup();
    if pids.is_empty() {
        return;
    }

    let joined = pids.join(",");
    let Ok(output) = run_host("ps", &["-o", "pid=,pgid=", "-p", &joined]).await else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let groups = parse_ps_pgid_output(&stdout);
    for entry in entries.iter_mut() {
        let Some(pid) = entry.pid else {
            continue;
        };
        entry.pgid = groups
            .iter()
            .find(|(candidate, _)| *candidate == pid)
            .map(|(_, pgid)| *pgid);
    }
}

/// Signal a process group (`kill -TERM -- -<pgid>`), or a single PID when unknown.
pub async fn kill_target(target: SignalTarget, force: bool) -> Result<(), String> {
    let signal = if force { "-KILL" } else { "-TERM" };
    let arg = target.kill_arg();
    // `--` keeps the leading `-` of a group id from being read as a flag.
    let output = run_host("kill", &[signal, "--", &arg]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_kill_failure(&stderr))
}

pub async fn kill_then_list(target: SignalTarget, force: bool) -> Result<Vec<PortEntry>, String> {
    kill_target(target, force).await?;
    tokio::time::sleep(KILL_SETTLE).await;
    list_ports().await
}

/// SIGTERM every listed target, then reload.
///
/// One failing signal must not abort the sweep: a process that already exited (or refuses
/// the signal) still leaves the rest to clean up.
pub async fn kill_all_then_list(targets: Vec<SignalTarget>) -> Result<Vec<PortEntry>, String> {
    for target in targets {
        let _ = kill_target(target, false).await;
    }
    tokio::time::sleep(KILL_SETTLE).await;
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
