//! Docker volumes CLI.

use crate::{VolumeSummary, parse_volumes_output};

use super::docker::{format_cli_failure_public, reclaimed_summary, run_docker};

const VOLUMES_FORMAT: &str = "{{.Name}}|{{.Driver}}";

/// List volumes (`docker volume ls`).
pub async fn list_volumes() -> Result<Vec<VolumeSummary>, String> {
    let output = run_docker(&["volume", "ls", "--format", VOLUMES_FORMAT]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure_public(
            &stderr,
            "docker volume ls failed",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_volumes_output(&stdout))
}

/// Remove a volume by name (`docker volume rm`).
pub async fn remove_volume(name: &str) -> Result<(), String> {
    let output = run_docker(&["volume", "rm", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure_public(
        &stderr,
        "docker volume rm failed",
    ))
}

pub async fn remove_then_list(name: String) -> Result<Vec<VolumeSummary>, String> {
    remove_volume(&name).await?;
    list_volumes().await
}

/// Remove every volume not attached to a container (`docker volume prune -a -f`).
///
/// Without `-a` Docker only prunes *anonymous* volumes, leaving unused named ones behind.
/// Docker older than 23.0 has no `-a` for this command, so fall back to the plain form.
pub async fn prune_volumes() -> Result<String, String> {
    let mut output = run_docker(&["volume", "prune", "-a", "-f"]).await?;
    if !output.status.success() && is_unknown_flag(&String::from_utf8_lossy(&output.stderr)) {
        output = run_docker(&["volume", "prune", "-f"]).await?;
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure_public(
            &stderr,
            "docker volume prune failed",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(reclaimed_summary(&stdout, "Unused volumes removed"))
}

fn is_unknown_flag(stderr: &str) -> bool {
    stderr.contains("unknown flag") || stderr.contains("unknown shorthand flag")
}
