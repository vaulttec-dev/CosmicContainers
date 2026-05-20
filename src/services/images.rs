//! Docker images CLI.

use crate::{ImageSummary, parse_images_output};

use super::docker::{format_cli_failure_public, run_docker};

const IMAGES_FORMAT: &str = "{{.Repository}}:{{.Tag}}|{{.Size}}|{{.ID}}";

/// List local images (`docker images`).
pub async fn list_images() -> Result<Vec<ImageSummary>, String> {
    let output = run_docker(&["images", "--format", IMAGES_FORMAT]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure_public(&stderr, "docker images failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_images_output(&stdout))
}
