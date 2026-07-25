use cosmic_containers::{ContainerSummary, parse_ps_output};

#[test]
fn display_label_without_service_uses_name() {
    let list = parse_ps_output("mybox|running|proj\n");
    assert_eq!(list[0].display_label(), "mybox");
}

#[test]
fn is_running_matches_running_prefix() {
    let list = parse_ps_output("a|running (healthy)|p|s\nb|exited (0)|p|s\n");
    assert!(list[0].is_running());
    assert!(!list[1].is_running());
}

#[test]
fn container_summary_equality_includes_all_fields() {
    let a = ContainerSummary {
        name: "n".into(),
        state: "running".into(),
        compose_project: Some("p".into()),
        compose_service: Some("web".into()),
        ports: None,
    };
    let b = a.clone();
    assert_eq!(a, b);
}
