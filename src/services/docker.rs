//! Docker CLI integration (no HTTP socket client).

use crate::{ContainerSummary, parse_ps_output};

const PS_FORMAT: &str = "{{.Names}}|{{.State}}";

fn format_spawn_error(err: &std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::NotFound {
        return "Docker not found. Install Docker and ensure `docker` is in PATH.".into();
    }
    format!("Failed to run docker: {err}")
}

fn format_cli_failure(stderr: &str, hint: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return hint.to_string();
    }
    if stderr.contains("permission denied")
        || stderr.contains("Cannot connect to the Docker daemon")
    {
        return format!(
            "{stderr}\n\nHint: add your user to the `docker` group or use rootless Docker."
        );
    }
    stderr.to_string()
}

async fn run_docker(args: &[&str]) -> Result<std::process::Output, String> {
    tokio::process::Command::new("docker")
        .args(args)
        .output()
        .await
        .map_err(|e| format_spawn_error(&e))
}

/// List all containers (`docker ps -a`).
pub async fn list_containers() -> Result<Vec<ContainerSummary>, String> {
    let output = run_docker(&["ps", "-a", "--format", PS_FORMAT]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure(
            &stderr,
            "docker ps failed. Is the Docker daemon running?",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_ps_output(&stdout))
}

/// Start a container by name.
pub async fn start_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["start", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker start failed"))
}

/// Stop a container by name.
pub async fn stop_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["stop", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker stop failed"))
}

/// Start or stop, then refresh the container list.
pub async fn start_then_list(name: String) -> Result<Vec<ContainerSummary>, String> {
    start_container(&name).await?;
    list_containers().await
}

/// Stop, then refresh the container list.
pub async fn stop_then_list(name: String) -> Result<Vec<ContainerSummary>, String> {
    stop_container(&name).await?;
    list_containers().await
}
