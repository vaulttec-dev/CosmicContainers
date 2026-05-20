//! Docker volumes CLI.

use crate::{VolumeSummary, parse_volumes_output};

use super::docker::{format_cli_failure_public, run_docker};

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
