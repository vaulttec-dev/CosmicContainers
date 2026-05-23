use linux_pop_extension::action_icons;

fn assert_svg(bytes: &[u8]) {
    let text = std::str::from_utf8(bytes).expect("utf-8 svg");
    assert!(text.contains("<svg"), "expected svg root element");
    assert!(text.contains("currentColor"), "expected theme-aware svg");
}

#[test]
fn embedded_action_icons_are_valid_svg() {
    assert_svg(action_icons::PLAY);
    assert_svg(action_icons::STOP);
    assert_svg(action_icons::RELOAD);
    assert_svg(action_icons::DELETE);
}
