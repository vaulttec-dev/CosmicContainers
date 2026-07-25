//! Parse `docker` CLI stdout into domain types.

use crate::types::{ContainerStats, ContainerSummary, ImageSummary, VolumeSummary};

/// Parse one `name|state|project|service|ports` line from `docker ps --format`.
pub fn parse_ps_line(line: &str) -> Option<ContainerSummary> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let mut parts = line.split('|');
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }
    let state = parts.next()?.trim().to_string();
    let compose_project = optional_field(parts.next());
    let compose_service = optional_field(parts.next());
    let ports = optional_field(parts.next());
    Some(ContainerSummary {
        name: name.to_string(),
        state,
        compose_project,
        compose_service,
        ports,
    })
}

fn optional_field(value: Option<&str>) -> Option<String> {
    value.map(str::trim).and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    })
}

/// Parse `docker ps` stdout.
pub fn parse_ps_output(stdout: &str) -> Vec<ContainerSummary> {
    stdout.lines().filter_map(parse_ps_line).collect()
}

pub fn parse_images_line(line: &str) -> Option<ImageSummary> {
    let line = line.trim();
    let mut parts = line.split('|');
    let reference = parts.next()?.trim();
    let size = parts.next()?.trim();
    let id = parts.next()?.trim();
    if reference.is_empty() || size.is_empty() || id.is_empty() {
        return None;
    }
    Some(ImageSummary {
        reference: reference.to_string(),
        size: size.to_string(),
        id: id.to_string(),
    })
}

pub fn parse_images_output(stdout: &str) -> Vec<ImageSummary> {
    stdout.lines().filter_map(parse_images_line).collect()
}

pub fn parse_volumes_line(line: &str) -> Option<VolumeSummary> {
    let line = line.trim();
    let (name, driver) = line.split_once('|')?;
    if name.is_empty() {
        return None;
    }
    Some(VolumeSummary {
        name: name.to_string(),
        driver: driver.trim().to_string(),
    })
}

pub fn parse_volumes_output(stdout: &str) -> Vec<VolumeSummary> {
    stdout.lines().filter_map(parse_volumes_line).collect()
}

/// Parse one `/name|8080->80/tcp, ` line from `docker inspect --format`.
pub fn parse_inspect_ports_line(line: &str) -> Option<(String, String)> {
    let (name, ports) = line.trim().split_once('|')?;
    let name = name.trim().trim_start_matches('/');
    let ports = ports.trim().trim_end_matches(',').trim();
    if name.is_empty() || ports.is_empty() {
        return None;
    }
    Some((name.to_string(), ports.to_string()))
}

/// Configured port bindings per container name (`docker ps` omits these when stopped).
pub fn parse_inspect_ports_output(stdout: &str) -> Vec<(String, String)> {
    stdout
        .lines()
        .filter_map(parse_inspect_ports_line)
        .collect()
}

/// Parse one `name|cpu|mem used / mem limit` line from `docker stats --format`.
pub fn parse_stats_line(line: &str) -> Option<ContainerStats> {
    let line = line.trim();
    let mut parts = line.split('|');
    let name = parts.next()?.trim();
    let cpu = parts.next()?.trim();
    let memory = parts.next()?.trim();
    if name.is_empty() || cpu.is_empty() || memory.is_empty() {
        return None;
    }
    // `docker stats` reports memory as `used / limit`; the popup only shows usage.
    let memory = memory
        .split_once('/')
        .map_or(memory, |(used, _)| used.trim());
    Some(ContainerStats {
        name: name.to_string(),
        cpu: cpu.to_string(),
        memory: memory.to_string(),
    })
}

pub fn parse_stats_output(stdout: &str) -> Vec<ContainerStats> {
    stdout.lines().filter_map(parse_stats_line).collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_images_output, parse_ps_output, parse_stats_output};

    #[test]
    fn parse_ps_output_reads_ports() {
        let stdout = "web|running|app|web|0.0.0.0:8080->80/tcp, :::8080->80/tcp\n";
        let list = parse_ps_output(stdout);
        assert_eq!(list[0].port_summary().as_deref(), Some("8080→80"));
    }

    #[test]
    fn parse_inspect_ports_output_reads_configured_bindings() {
        let stdout = "/web|8080->80/tcp, 5432->5432/tcp, \n/bare|\n";
        let ports = super::parse_inspect_ports_output(stdout);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].0, "web");
        assert_eq!(ports[0].1, "8080->80/tcp, 5432->5432/tcp");
    }

    #[test]
    fn parse_stats_output_strips_memory_limit() {
        let stdout = "web|1.23%|45.2MiB / 7.66GiB\n";
        let list = parse_stats_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].memory, "45.2MiB");
        assert_eq!(list[0].usage_line(), "CPU 1.23% · RAM 45.2MiB");
    }

    #[test]
    fn parse_ps_output_parses_compose_service() {
        let stdout = "postgres|running|doc2pay|postgres\norphan|created||\n";
        let list = parse_ps_output(stdout);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].compose_service.as_deref(), Some("postgres"));
        assert_eq!(list[0].display_label(), "postgres (postgres)");
        assert_eq!(list[1].compose_service, None);
    }

    #[test]
    fn parse_ps_output_legacy_three_fields() {
        let stdout = "only-name|running|proj\n";
        let list = parse_ps_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].compose_project.as_deref(), Some("proj"));
        assert_eq!(list[0].compose_service, None);
    }

    #[test]
    fn parse_images_output_parses_lines() {
        let stdout = "redis:7|57MB|abc123\n";
        let list = parse_images_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].reference, "redis:7");
    }
}
