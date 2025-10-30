//! Phase 1 machine encoding helpers.
//!
//! Characters are written into a machine texture by encoding the ASCII code in
//! the red channel. The compute shader expands these into human-readable
//! glyphs.

use crate::glyph_rom::GLYPH_ROM;

pub const FIRST_PRINTABLE: u8 = 32;
pub const LAST_PRINTABLE: u8 = 126;
pub const GLYPH_WIDTH: u32 = 5;
pub const GLYPH_HEIGHT: u32 = 7;
pub const GLYPH_HEIGHT_USIZE: usize = GLYPH_HEIGHT as usize;

/// Interface used by the glyph bootstrap helpers to write map pixels.
#[allow(dead_code)]
pub trait MapInterface {
    fn set_pixel(&mut self, pos: (u32, u32), color: [u8; 3]);
    fn advance_cursor(&mut self);
}

/// Translate a typed character into its encoded RGB colour.
///
/// The bootstrap supports ASCII input; red holds the byte, other channels stay
/// zero so that compute shaders can decode by reading `color.r`.
#[allow(dead_code)]
pub fn char_to_color(ch: char) -> Option<[u8; 3]> {
    if ch.is_ascii() && !ch.is_ascii_control() {
        Some([ch as u8, 0, 0])
    } else {
        None
    }
}

/// Write the encoded pixel for the provided character.
#[allow(dead_code)]
pub fn on_key_press(
    ch: char,
    cursor_pos: (u32, u32),
    map_interface: &mut dyn MapInterface,
) -> Option<[u8; 3]> {
    if let Some(color) = char_to_color(ch) {
        map_interface.set_pixel(cursor_pos, color);
        map_interface.advance_cursor();
        Some(color)
    } else {
        None
    }
}

/// Fetch the 5x7 glyph row data for a printable ASCII character.
///
/// Each row uses the lowest five bits to represent pixels from left to right.
pub fn glyph_rows(ascii: u8) -> Option<&'static [u8; GLYPH_HEIGHT_USIZE]> {
    if ascii < FIRST_PRINTABLE || ascii > LAST_PRINTABLE {
        None
    } else {
        Some(&GLYPH_ROM[(ascii - FIRST_PRINTABLE) as usize])
    }
}
