//! Filter and group container lists for the popup UI.

use crate::types::{ContainerGroup, ContainerSummary, STANDALONE_GROUP};

/// Keep only running containers when `running_only` is true.
pub fn filter_containers(
    containers: Vec<ContainerSummary>,
    running_only: bool,
) -> Vec<ContainerSummary> {
    if running_only {
        containers.into_iter().filter(|c| c.is_running()).collect()
    } else {
        containers
    }
}

/// Group containers by Compose project; projects sorted A–Z, standalone last.
pub fn group_containers(containers: Vec<ContainerSummary>) -> Vec<ContainerGroup> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, Vec<ContainerSummary>> = BTreeMap::new();
    for container in containers {
        map.entry(container.group_label().to_string())
            .or_default()
            .push(container);
    }

    let mut groups: Vec<ContainerGroup> = map
        .into_iter()
        .map(|(project, containers)| ContainerGroup {
            project,
            containers,
        })
        .collect();

    groups.sort_by(|a, b| {
        let a_standalone = a.project == STANDALONE_GROUP;
        let b_standalone = b.project == STANDALONE_GROUP;
        match (a_standalone, b_standalone) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.project.cmp(&b.project),
        }
    });

    for group in &mut groups {
        group.containers.sort_by_key(|c| c.display_label());
    }

    groups
}

/// Count containers whose state starts with `running`.
pub fn running_count(containers: &[ContainerSummary]) -> usize {
    containers.iter().filter(|c| c.is_running()).count()
}

#[cfg(test)]
mod tests {
    use super::{filter_containers, group_containers, running_count};
    use crate::parse::parse_ps_output;
    use crate::types::{ContainerGroup, STANDALONE_GROUP};

    #[test]
    fn filter_containers_running_only() {
        let all = parse_ps_output("a|running|p\nb|exited|p\n");
        let filtered = filter_containers(all, true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "a");
    }

    #[test]
    fn group_containers_sorts_projects_and_standalone_last() {
        let containers = parse_ps_output("b|running|zebra|s\na|exited|cafe|s\nc|running||\n");
        let groups = group_containers(containers);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[2].project, STANDALONE_GROUP);
    }

    #[test]
    fn running_count_counts_only_running() {
        let stdout = "a|running|p|s\nb|exited|p|s\nc|running|p|s\n";
        let list = parse_ps_output(stdout);
        assert_eq!(running_count(&list), 2);
    }

    #[test]
    fn container_group_running_count() {
        let group = ContainerGroup {
            project: "app".into(),
            containers: parse_ps_output("a|running|app|web\nb|exited|app|api\n"),
        };
        assert_eq!(group.running_count(), 1);
        assert!(group.has_running());
        assert!(group.has_stopped());
    }
}
