//! Parse `docker` CLI stdout into domain types.

use crate::types::{ContainerStats, ContainerSummary, ImageSummary, PortEntry, VolumeSummary};

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

/// Parse one `ss -tulpnH` row:
/// `tcp LISTEN 0 4096 0.0.0.0:8080 0.0.0.0:* users:(("nginx",pid=42,fd=6))`.
pub fn parse_ss_line(line: &str) -> Option<PortEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    // Only the first five columns are positional; `users:(…)` may contain spaces
    // (`"next-server (v1"`), so it is read from the raw line instead.
    let mut columns = line.split_whitespace();
    let protocol = columns.next()?.to_string();
    let _state = columns.next()?;
    let _recv_q = columns.next()?;
    let _send_q = columns.next()?;
    let (address, port) = split_socket_address(columns.next()?)?;
    let (process, pid) = line
        .split_once("users:(")
        .map_or((None, None), |(_, users)| parse_ss_users(users));
    Some(PortEntry {
        port,
        protocol,
        addresses: vec![address],
        process,
        pid,
        pgid: None,
    })
}

/// Parse `ps -o pid=,pgid=` stdout into `(pid, pgid)` pairs.
pub fn parse_ps_pgid_output(stdout: &str) -> Vec<(u32, u32)> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut columns = line.split_whitespace();
            let pid = columns.next()?.parse().ok()?;
            let pgid = columns.next()?.parse().ok()?;
            Some((pid, pgid))
        })
        .collect()
}

/// `127.0.0.1:6379` → `(127.0.0.1, 6379)`; also handles `[::]:80` and `*:3000`.
fn split_socket_address(local: &str) -> Option<(String, u16)> {
    let (address, port) = local.rsplit_once(':')?;
    let port = port.parse().ok()?;
    Some((address.to_string(), port))
}

/// `("inngest",pid=2271470,fd=10))` → `(Some("inngest"), Some(2271470))`.
fn parse_ss_users(users: &str) -> (Option<String>, Option<u32>) {
    let process = users
        .split('"')
        .nth(1)
        .filter(|name| !name.is_empty())
        .map(clean_process_name);
    let pid = users.split_once("pid=").and_then(|(_, tail)| {
        tail.chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    });
    (process, pid)
}

/// `ss` reports the kernel `comm`, capped at 15 bytes, so a process title like
/// `next-server (v15.0.3)` arrives cut off as `next-server (v1`. Drop the mangled tail.
fn clean_process_name(name: &str) -> String {
    match name.split_once(" (") {
        Some((head, _)) if !head.is_empty() => head.to_string(),
        _ => name.to_string(),
    }
}

/// Parse `ss` stdout, merging the IPv4/IPv6 rows of one port and sorting by port.
pub fn parse_ss_output(stdout: &str) -> Vec<PortEntry> {
    let mut entries: Vec<PortEntry> = Vec::new();
    for entry in stdout.lines().filter_map(parse_ss_line) {
        let existing = entries.iter_mut().find(|candidate| {
            candidate.port == entry.port
                && candidate.protocol == entry.protocol
                && candidate.pid == entry.pid
        });
        match existing {
            Some(existing) => {
                for address in entry.addresses {
                    if !existing.addresses.contains(&address) {
                        existing.addresses.push(address);
                    }
                }
            }
            None => entries.push(entry),
        }
    }
    entries.sort_by(|a, b| {
        a.port
            .cmp(&b.port)
            .then_with(|| a.protocol.cmp(&b.protocol))
    });
    entries
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
    fn parse_ps_output_detects_published_host_port() {
        let stdout = "web|running|app|web|0.0.0.0:8080->80/tcp, :::8080->80/tcp\n";
        let list = parse_ps_output(stdout);
        assert!(list[0].publishes_host_port(8080));
        assert!(!list[0].publishes_host_port(80));
    }

    #[test]
    fn parse_ps_output_detects_published_host_port_range() {
        let stdout = "minio|running|app|minio|0.0.0.0:9000-9001->9000-9001/tcp\n";
        let list = parse_ps_output(stdout);
        assert!(list[0].publishes_host_port(9000));
        assert!(list[0].publishes_host_port(9001));
        assert!(!list[0].publishes_host_port(9002));
    }

    #[test]
    fn parse_ss_output_reads_process_names_with_spaces() {
        let stdout =
            "tcp LISTEN 0 511 *:3000 *:* users:((\"next-server (v1\",pid=2271194,fd=22))\n";
        let list = super::parse_ss_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].port, 3000);
        assert_eq!(list[0].protocol, "tcp");
        // The 15-byte `comm` cap cuts `next-server (v15.0.3)`; the tail is dropped.
        assert_eq!(list[0].process.as_deref(), Some("next-server"));
        assert_eq!(list[0].pid, Some(2271194));
        assert_eq!(list[0].owner_line(), "next-server · PID 2271194");
    }

    #[test]
    fn parse_ss_output_merges_ipv4_and_ipv6_rows() {
        let stdout =
            "tcp LISTEN 0 4096 0.0.0.0:9000 0.0.0.0:*\ntcp LISTEN 0 4096 [::]:9000 [::]:*\n";
        let list = super::parse_ss_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].addresses, vec!["0.0.0.0", "[::]"]);
        assert_eq!(list[0].pid, None);
        assert_eq!(
            list[0].owner_line(),
            "Another user (root) · needs privileges to stop"
        );
    }

    #[test]
    fn parse_ss_output_sorts_by_port_and_keeps_udp() {
        let stdout = "udp UNCONN 0 0 0.0.0.0:5353 0.0.0.0:*\ntcp LISTEN 0 511 *:80 *:*\n";
        let list = super::parse_ss_output(stdout);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].title(), "80/tcp");
        assert_eq!(list[1].title(), "5353/udp");
    }

    #[test]
    fn parse_ps_pgid_output_reads_pairs() {
        let stdout = "   9411    9292\n   9356    8538\ngarbage\n";
        let pairs = super::parse_ps_pgid_output(stdout);
        assert_eq!(pairs, vec![(9411, 9292), (9356, 8538)]);
    }

    /// A killed listener is respawned by its supervisor, so the group is the real target.
    #[test]
    fn signal_target_prefers_the_process_group() {
        use crate::types::SignalTarget;

        let stdout = "tcp LISTEN 0 511 *:3000 *:* users:((\"node\",pid=9411,fd=22))\n";
        let mut list = super::parse_ss_output(stdout);
        assert_eq!(list[0].signal_target(), Some(SignalTarget::Process(9411)));

        list[0].pgid = Some(9292);
        assert_eq!(list[0].signal_target(), Some(SignalTarget::Group(9292)));
        assert_eq!(list[0].signal_target().unwrap().kill_arg(), "-9292");

        // PGID 1 means the parent already died and init adopted it — never signal that.
        list[0].pgid = Some(1);
        assert_eq!(list[0].signal_target(), Some(SignalTarget::Process(9411)));
    }

    /// Every shape of dev server survives the filter; system noise does not.
    #[test]
    fn retain_dev_servers_covers_every_launch_shape() {
        let stdout = concat!(
            // ordinary host dev server
            "tcp LISTEN 0 511 *:3000 *:* users:((\"node\",pid=42,fd=22))\n",
            // livekit-server --dev: TCP signalling plus its own RTC socket on UDP
            "tcp LISTEN 0 511 *:7880 *:* users:((\"livekit-server\",pid=55,fd=8))\n",
            "udp UNCONN 0 0 0.0.0.0:7882 0.0.0.0:* users:((\"livekit-server\",pid=55,fd=9))\n",
            // started with sudo: root owns it, `ss` hides the pid, port is above 1024
            "tcp LISTEN 0 511 0.0.0.0:8443 0.0.0.0:*\n",
            // system services: root below 1024, and UDP-only browser mDNS
            "tcp LISTEN 0 4096 127.0.0.1:631 0.0.0.0:*\n",
            "udp UNCONN 0 0 224.0.0.251:5353 0.0.0.0:* users:((\"chrome\",pid=77,fd=9))\n",
        );
        let mut list = super::parse_ss_output(stdout);
        assert_eq!(list.len(), 6);

        crate::types::retain_dev_servers(&mut list);
        let kept: Vec<String> = list.iter().map(|entry| entry.title()).collect();
        assert_eq!(kept, vec!["3000/tcp", "7880/tcp", "7882/udp", "8443/tcp"]);

        // The sudo-started row is visible but cannot be signalled.
        let sudo_row = list.iter().find(|entry| entry.port == 8443).unwrap();
        assert!(!sudo_row.is_killable());
        assert_eq!(sudo_row.signal_target(), None);
    }

    #[test]
    fn parse_images_output_parses_lines() {
        let stdout = "redis:7|57MB|abc123\n";
        let list = parse_images_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].reference, "redis:7");
    }
}
