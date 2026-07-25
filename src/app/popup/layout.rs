//! Popup spacing, typography, and size tokens (SnakeFlow magic-number hygiene).

/// Fixed popup window size (all tabs use the same dimensions to avoid resize jumps).
pub const POPUP_WIDTH: f32 = 800.0;
pub const POPUP_HEIGHT: f32 = 960.0;

pub const SPACING_NONE: u16 = 0;
pub const SPACING_XXS: u16 = 2;
pub const SPACING_XS: u16 = 4;
pub const SPACING_SM: u16 = 6;
pub const SPACING_MD: u16 = 8;
pub const SPACING_LG: u16 = 10;

pub const FONT_STATUS: u16 = 12;
pub const FONT_SECONDARY: u16 = 11;
pub const FONT_PRIMARY: u16 = 13;

pub const ICON_SIZE: u16 = 16;

pub const PAD_HORIZONTAL: u16 = 12;

pub const PAD_POPUP: [u16; 4] = [SPACING_LG, PAD_HORIZONTAL, PAD_HORIZONTAL, PAD_HORIZONTAL];
pub const PAD_DIVIDER: [u16; 2] = [SPACING_SM, 0];
pub const PAD_STATUS: [u16; 4] = [0, 0, SPACING_XS, 0];
pub const PAD_TAB: [u16; 4] = [SPACING_SM, SPACING_LG, SPACING_SM, SPACING_LG];
pub const PAD_ICON_BUTTON: u16 = SPACING_XS;
pub const PAD_LIST_ROW: [u16; 4] = [SPACING_SM, SPACING_XS, SPACING_SM, SPACING_XS];
pub const PAD_GROUP_HEADER: [u16; 4] = [SPACING_MD, SPACING_XS, SPACING_XS, SPACING_XS];
pub const PAD_SECTION_DIVIDER: [u16; 4] = [SPACING_XS, SPACING_MD, SPACING_XS, SPACING_MD];
pub const PAD_CONFIRM_BUTTON: [u16; 4] = [SPACING_XS, SPACING_MD, SPACING_XS, SPACING_MD];

pub const FILL_PORTION_LABELS: u16 = 3;
pub const FILL_PORTION_PROMPT: u16 = 2;
