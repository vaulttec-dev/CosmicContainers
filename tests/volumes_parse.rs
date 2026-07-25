use cosmic_containers::parse_volumes_output;

#[test]
fn parse_volumes_output_skips_empty_names() {
    let stdout = "data|local\n|local\nvol|local\n";
    let list = parse_volumes_output(stdout);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "data");
    assert_eq!(list[1].name, "vol");
}

#[test]
fn parse_volumes_output_trims_driver() {
    let stdout = "cache| local \n";
    let list = parse_volumes_output(stdout);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].driver, "local");
}
