#[test]
fn project_constants() {
    assert_eq!(linux_pop_extension::APP_ID, "dev.linuxpop.Extension");
    assert_eq!(linux_pop_extension::PANEL_EMOJI, "😀");
    assert!(!linux_pop_extension::POPUP_GREETING.is_empty());
}
