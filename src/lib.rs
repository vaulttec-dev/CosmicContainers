//! Core types and Docker CLI helpers for the Cosmic Containers panel applet.

pub mod action_icons;
pub mod grouping;
pub mod parse;
pub mod services;
pub mod settings;
pub mod types;

pub use grouping::{group_containers, running_count};
pub use parse::{
    parse_images_output, parse_inspect_ports_output, parse_ps_output, parse_ss_output,
    parse_stats_output, parse_volumes_output,
};
pub use settings::{
    DEFAULT_PANEL_COUNT_FONT_SIZE, MAX_PANEL_COUNT_FONT_SIZE, MIN_PANEL_COUNT_FONT_SIZE,
    UserSettings,
};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::APP_ID;

    #[test]
    fn app_id_is_valid_rdns() {
        assert!(APP_ID.contains('.'));
        assert!(!APP_ID.is_empty());
    }
}
