//! Docker CLI integration (no HTTP socket client).

use crate::{
    ContainerGroup, ContainerSummary, STANDALONE_GROUP, parse_inspect_ports_output, parse_ps_output,
};

const PS_FORMAT: &str = r#"{{.Names}}|{{.State}}|{{.Label "com.docker.compose.project"}}|{{.Label "com.docker.compose.service"}}|{{.Ports}}"#;

/// Run a host binary, escaping the sandbox when packaged as a Flatpak.
pub(crate) async fn run_host(program: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = if std::env::var_os("FLATPAK_ID").is_some() {
        let mut cmd = tokio::process::Command::new("flatpak-spawn");
        cmd.arg("--host").arg(program);
        cmd
    } else {
        tokio::process::Command::new(program)
    };
    command
        .args(args)
        .output()
        .await
        .map_err(|e| format_spawn_error(program, &e))
}

pub(crate) async fn run_docker(args: &[&str]) -> Result<std::process::Output, String> {
    run_host("docker", args).await
}

pub(crate) fn format_cli_failure_public(stderr: &str, hint: &str) -> String {
    format_cli_failure(stderr, hint)
}

/// Summary line reported back to the popup after a prune (`docker` prints reclaimed space).
pub(crate) fn reclaimed_summary(stdout: &str, fallback: &str) -> String {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Total reclaimed space"))
        .map_or_else(|| fallback.to_string(), str::to_string)
}

fn format_spawn_error(program: &str, err: &std::io::Error) -> String {
    if err.kind() == std::io::ErrorKind::NotFound {
        return match program {
            "docker" => "Docker not found. Install Docker and ensure `docker` is in PATH.".into(),
            other => format!("`{other}` not found in PATH."),
        };
    }
    format!("Failed to run {program}: {err}")
}

fn format_cli_failure(stderr: &str, hint: &str) -> String {
    let stderr = stderr.trim();
    if stderr.is_empty() {
        return hint.to_string();
    }
    if stderr.contains("org.freedesktop.Flatpak") {
        return format!(
            "{stderr}\n\nHint: reinstall the Flatpak build with `--talk-name=org.freedesktop.Flatpak` in the manifest."
        );
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
    let mut containers = parse_ps_output(&stdout);
    fill_configured_ports(&mut containers).await;
    Ok(containers)
}

const INSPECT_PORTS_FORMAT: &str = "{{.Name}}|{{range $port, $bindings := .HostConfig.PortBindings}}{{range $bindings}}{{.HostPort}}->{{$port}}, {{end}}{{end}}";

/// `docker ps` reports ports only while a container runs; fill the rest from its config.
///
/// Best effort: a failing inspect just leaves those rows without a port line.
async fn fill_configured_ports(containers: &mut [ContainerSummary]) {
    let pending: Vec<String> = containers
        .iter()
        .filter(|container| container.ports.is_none())
        .map(|container| container.name.clone())
        .collect();
    if pending.is_empty() {
        return;
    }

    let mut args = vec!["inspect", "--format", INSPECT_PORTS_FORMAT];
    args.extend(pending.iter().map(String::as_str));
    let Ok(output) = run_docker(&args).await else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ports = parse_inspect_ports_output(&stdout);
    for container in containers.iter_mut() {
        if container.ports.is_some() {
            continue;
        }
        if let Some((_, bindings)) = ports.iter().find(|(name, _)| *name == container.name) {
            container.ports = Some(bindings.clone());
        }
    }
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

/// Restart a container by name.
pub async fn restart_container(name: &str) -> Result<(), String> {
    let output = run_docker(&["restart", name]).await?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format_cli_failure(&stderr, "docker restart failed"))
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
        "docker compose start failed for this project",
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
        "docker compose stop failed for this project",
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
        "docker compose restart failed for this project",
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
    Err(format_cli_failure(&stderr, "docker rm failed"))
}

pub async fn remove_then_list(name: String) -> Result<Vec<ContainerSummary>, String> {
    remove_container(&name).await?;
    list_containers().await
}

/// Remove every stopped container (`docker container prune -f`).
pub async fn prune_containers() -> Result<String, String> {
    let output = run_docker(&["container", "prune", "-f"]).await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format_cli_failure(&stderr, "docker container prune failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(reclaimed_summary(&stdout, "Stopped containers removed"))
}
