use linux_pop_extension::parse_images_output;

#[test]
fn parse_images_output_skips_incomplete_lines() {
    let stdout = "redis:7|57MB|abc\nincomplete|only\nnginx|12MB|def456\n";
    let list = parse_images_output(stdout);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].reference, "redis:7");
    assert_eq!(list[1].id, "def456");
}

#[test]
fn parse_images_output_preserves_size() {
    let stdout = "alpine:latest|7.05MB|sha256:aaa\n";
    let list = parse_images_output(stdout);
    assert_eq!(list[0].size, "7.05MB");
}
