//! Core constants for the linux-pop-extension COSMIC panel applet.

/// Desktop entry / application ID (reverse domain notation).
pub const APP_ID: &str = "dev.linuxpop.Extension";

/// Emoji shown on the panel for the starter widget.
pub const PANEL_EMOJI: &str = "😀";

#[cfg(test)]
mod tests {
    use super::{APP_ID, PANEL_EMOJI};

    #[test]
    fn app_id_is_valid_rdns() {
        assert!(APP_ID.contains('.'));
        assert!(!APP_ID.is_empty());
    }

    #[test]
    fn panel_emoji_is_non_empty() {
        assert!(!PANEL_EMOJI.is_empty());
    }
}
