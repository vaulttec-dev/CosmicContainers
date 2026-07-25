//! Live container resource usage (`docker stats`).

use crate::{ContainerStats, parse_stats_output};

use super::docker::{format_cli_failure_public, run_docker};

const STATS_FORMAT: &str = "{{.Name}}|{{.CPUPerc}}|{{.MemUsage}}";

/// Snapshot CPU/memory for running containers (`docker stats --no-stream`).
pub async fn list_stats() -> Result<Vec<ContainerStats>, String> {
    let output = run_docker(&["stats", "--no-stream", "--format", STATS_FORMAT]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure_public(&stderr, "docker stats failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_stats_output(&stdout))
}
