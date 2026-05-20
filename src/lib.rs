//! Core types and Docker CLI helpers for the Cosmic Containers panel applet.

pub mod services;

/// Desktop entry / application ID (reverse domain notation).
pub const APP_ID: &str = "dev.linuxpop.Extension";

/// One row from `docker ps --format "{{.Names}}|{{.State}}"`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContainerSummary {
    pub name: String,
    pub state: String,
}

impl ContainerSummary {
    pub fn is_running(&self) -> bool {
        self.state.starts_with("running")
    }
}

/// Parse `docker ps` stdout (one `name|state` per line).
pub fn parse_ps_output(stdout: &str) -> Vec<ContainerSummary> {
    stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (name, state) = line.split_once('|')?;
            if name.is_empty() {
                return None;
            }
            Some(ContainerSummary {
                name: name.to_string(),
                state: state.to_string(),
            })
        })
        .collect()
}

/// Count containers whose state starts with `running`.
pub fn running_count(containers: &[ContainerSummary]) -> usize {
    containers.iter().filter(|c| c.is_running()).count()
}

#[cfg(test)]
mod tests {
    use super::{APP_ID, ContainerSummary, parse_ps_output, running_count};

    #[test]
    fn app_id_is_valid_rdns() {
        assert!(APP_ID.contains('.'));
        assert!(!APP_ID.is_empty());
    }

    #[test]
    fn parse_ps_output_parses_lines() {
        let stdout = "postgres-dev|running\nredis|exited (0)\n";
        let list = parse_ps_output(stdout);
        assert_eq!(
            list,
            vec![
                ContainerSummary {
                    name: "postgres-dev".into(),
                    state: "running".into(),
                },
                ContainerSummary {
                    name: "redis".into(),
                    state: "exited (0)".into(),
                },
            ]
        );
    }

    #[test]
    fn parse_ps_output_skips_malformed_lines() {
        let stdout = "good|running\nbadline\n|empty\n";
        let list = parse_ps_output(stdout);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "good");
    }

    #[test]
    fn parse_ps_output_empty() {
        assert!(parse_ps_output("").is_empty());
        assert!(parse_ps_output("\n\n").is_empty());
    }

    #[test]
    fn running_count_counts_only_running() {
        let stdout = "a|running\nb|exited\nc|running\n";
        let list = parse_ps_output(stdout);
        assert_eq!(running_count(&list), 2);
    }
}
