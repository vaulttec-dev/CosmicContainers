//! Docker images CLI.

use crate::{ImageSummary, parse_images_output};

use super::docker::{format_cli_failure_public, reclaimed_summary, run_docker};

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

/// Remove an image by ID or reference (`docker rmi`).
pub async fn remove_image(id: &str) -> Result<(), String> {
    let output = run_docker(&["rmi", id]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure_public(&stderr, "docker rmi failed"))
}

pub async fn remove_then_list(id: String) -> Result<Vec<ImageSummary>, String> {
    remove_image(&id).await?;
    list_images().await
}

/// Remove every image not used by a container (`docker image prune -a -f`).
pub async fn prune_images() -> Result<String, String> {
    let output = run_docker(&["image", "prune", "-a", "-f"]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure_public(
            &stderr,
            "docker image prune failed",
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(reclaimed_summary(&stdout, "Unused images removed"))
}
