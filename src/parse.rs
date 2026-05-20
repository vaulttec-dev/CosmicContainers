//! Parse `docker` CLI stdout into domain types.

use crate::types::{ContainerSummary, ImageSummary, VolumeSummary};

/// Parse one `name|state|project|service` line from `docker ps --format`.
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
    Some(ContainerSummary {
        name: name.to_string(),
        state,
        compose_project,
        compose_service,
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

#[cfg(test)]
mod tests {
    use super::{parse_images_output, parse_ps_output};

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
