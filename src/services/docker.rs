//! Docker CLI integration (no HTTP socket client).

use crate::{ContainerGroup, ContainerSummary, STANDALONE_GROUP, parse_ps_output};

const PS_FORMAT: &str = r#"{{.Names}}|{{.State}}|{{.Label "com.docker.compose.project"}}|{{.Label "com.docker.compose.service"}}"#;

pub(crate) async fn run_docker(args: &[&str]) -> Result<std::process::Output, String> {
    tokio::process::Command::new("docker")
        .args(args)
        .output()
        .await
        .map_err(|e| format_spawn_error(&e))
}

pub(crate) fn format_cli_failure_public(stderr: &str, hint: &str) -> String {
    format_cli_failure(stderr, hint)
}

fn format_spawn_error(err: &std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::NotFound {
        return "Docker не знайдено. Встановіть Docker і переконайтеся, що `docker` є в PATH."
            .into();
    }
    format!("Не вдалося запустити docker: {err}")
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
            "{stderr}\n\nПідказка: додайте користувача до групи `docker` або використовуйте rootless Docker."
        );
    }
    stderr.to_string()
}

/// List all containers (`docker ps -a`).
pub async fn list_containers() -> Result<Vec<ContainerSummary>, String> {
    let output = run_docker(&["ps", "-a", "--format", PS_FORMAT]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure(
            &stderr,
            "docker ps не вдався. Чи запущений демон Docker?",
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
    Err(format_cli_failure(&stderr, "docker start не вдався"))
}

/// Stop a container by name.
pub async fn stop_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["stop", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker stop не вдався"))
}

/// Restart a container by name.
pub async fn restart_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["restart", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker restart не вдався"))
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

pub async fn restart_then_list(name: String) -> Result<Vec<ContainerSummary>, String> {
    restart_container(&name).await?;
    list_containers().await
}

async fn restart_containers_individually(containers: &[ContainerSummary]) -> Result<(), String> {
    for container in containers {
        restart_container(&container.name).await?;
    }
    Ok(())
}

async fn start_containers_individually(containers: &[ContainerSummary]) -> Result<(), String> {
    for container in containers {
        if !container.is_running() {
            start_container(&container.name).await?;
        }
    }
    Ok(())
}

async fn stop_containers_individually(containers: &[ContainerSummary]) -> Result<(), String> {
    for container in containers {
        if container.is_running() {
            stop_container(&container.name).await?;
        }
    }
    Ok(())
}

/// Start every container in a group (Compose project or standalone set).
pub async fn start_group(group: &ContainerGroup) -> Result<(), String> {
    if group.project == STANDALONE_GROUP {
        return start_containers_individually(&group.containers).await;
    }

    let output = run_docker(&["compose", "-p", &group.project, "start"]).await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("no configuration file provided")
        || stderr.contains("not found")
        || stderr.contains("No such")
    {
        return start_containers_individually(&group.containers).await;
    }

    Err(format_cli_failure(
        &stderr,
        "docker compose start не вдався для цього проєкту",
    ))
}

/// Stop every container in a group.
pub async fn stop_group(group: &ContainerGroup) -> Result<(), String> {
    if group.project == STANDALONE_GROUP {
        return stop_containers_individually(&group.containers).await;
    }

    let output = run_docker(&["compose", "-p", &group.project, "stop"]).await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("no configuration file provided")
        || stderr.contains("not found")
        || stderr.contains("No such")
    {
        return stop_containers_individually(&group.containers).await;
    }

    Err(format_cli_failure(
        &stderr,
        "docker compose stop не вдався для цього проєкту",
    ))
}

pub async fn start_group_then_list(group: ContainerGroup) -> Result<Vec<ContainerSummary>, String> {
    start_group(&group).await?;
    list_containers().await
}

pub async fn stop_group_then_list(group: ContainerGroup) -> Result<Vec<ContainerSummary>, String> {
    stop_group(&group).await?;
    list_containers().await
}

/// Restart every container in a group.
pub async fn restart_group(group: &ContainerGroup) -> Result<(), String> {
    if group.project == STANDALONE_GROUP {
        return restart_containers_individually(&group.containers).await;
    }

    let output = run_docker(&["compose", "-p", &group.project, "restart"]).await?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("no configuration file provided")
        || stderr.contains("not found")
        || stderr.contains("No such")
    {
        return restart_containers_individually(&group.containers).await;
    }

    Err(format_cli_failure(
        &stderr,
        "docker compose restart не вдався для цього проєкту",
    ))
}

pub async fn restart_group_then_list(
    group: ContainerGroup,
) -> Result<Vec<ContainerSummary>, String> {
    restart_group(&group).await?;
    list_containers().await
}

/// Remove a stopped container (`docker rm`).
pub async fn remove_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["rm", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker rm не вдався"))
}

pub async fn remove_then_list(name: String) -> Result<Vec<ContainerSummary>, String> {
    remove_container(&name).await?;
    list_containers().await
}
