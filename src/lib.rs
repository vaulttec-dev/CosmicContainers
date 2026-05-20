//! Core types and Docker CLI helpers for the Cosmic Containers panel applet.

pub mod action_icons;
pub mod grouping;
pub mod parse;
pub mod services;
pub mod types;

pub use grouping::{filter_containers, group_containers, running_count};
pub use parse::{parse_images_output, parse_ps_output, parse_volumes_output};
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
