//! Conversion utilities between Fusabi TUI and Scarab types.

use fusabi_tui_core::buffer::Cell as TuiCell;
use fusabi_tui_core::style::{Color, Modifier};

use crate::shared::SharedCell;

/// Convert a Fusabi Color to RGBA u32 format used by Scarab.
///
/// Scarab uses ARGB format: 0xAARRGGBB
pub fn color_to_u32(color: Color) -> u32 {
    match color {
        Color::Black => 0xFF000000,
        Color::Red => 0xFFFF0000,
        Color::Green => 0xFF00FF00,
        Color::Yellow => 0xFFFFFF00,
        Color::Blue => 0xFF0000FF,
        Color::Magenta => 0xFFFF00FF,
        Color::Cyan => 0xFF00FFFF,
        Color::White => 0xFFFFFFFF,
        Color::DarkGray => 0xFF808080,
        Color::LightRed => 0xFFFF8080,
        Color::LightGreen => 0xFF80FF80,
        Color::LightYellow => 0xFFFFFF80,
        Color::LightBlue => 0xFF8080FF,
        Color::LightMagenta => 0xFFFF80FF,
        Color::LightCyan => 0xFF80FFFF,
        Color::LightWhite => 0xFFFFFFFF,
        Color::Rgb(r, g, b) => {
            let a = 0xFF;
            ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }
        Color::Indexed(idx) => {
            // Convert indexed color to approximate RGB
            indexed_to_rgb(idx)
        }
        Color::Reset => 0xFFA8DF5A, // Default Slime green
    }
}

/// Convert RGBA u32 format to Fusabi Color.
pub fn u32_to_color(rgba: u32) -> Color {
    let r = ((rgba >> 16) & 0xFF) as u8;
    let g = ((rgba >> 8) & 0xFF) as u8;
    let b = (rgba & 0xFF) as u8;
    Color::Rgb(r, g, b)
}

/// Convert indexed color (0-255) to RGB.
fn indexed_to_rgb(idx: u8) -> u32 {
    match idx {
        // Standard colors (0-15)
        0 => 0xFF000000,  // Black
        1 => 0xFF800000,  // Maroon
        2 => 0xFF008000,  // Green
        3 => 0xFF808000,  // Olive
        4 => 0xFF000080,  // Navy
        5 => 0xFF800080,  // Purple
        6 => 0xFF008080,  // Teal
        7 => 0xFFC0C0C0,  // Silver
        8 => 0xFF808080,  // Gray
        9 => 0xFFFF0000,  // Red
        10 => 0xFF00FF00, // Lime
        11 => 0xFFFFFF00, // Yellow
        12 => 0xFF0000FF, // Blue
        13 => 0xFFFF00FF, // Fuchsia
        14 => 0xFF00FFFF, // Aqua
        15 => 0xFFFFFFFF, // White

        // 216 color cube (16-231)
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }

        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32)
        }
    }
}

/// Convert Fusabi Modifier to Scarab flags.
pub fn modifier_to_flags(modifier: Modifier) -> u8 {
    let mut flags = 0u8;

    if modifier.contains(Modifier::BOLD) {
        flags |= SharedCell::FLAG_BOLD;
    }
    if modifier.contains(Modifier::DIM) {
        flags |= SharedCell::FLAG_DIM;
    }
    if modifier.contains(Modifier::ITALIC) {
        flags |= SharedCell::FLAG_ITALIC;
    }
    if modifier.contains(Modifier::UNDERLINED) {
        flags |= SharedCell::FLAG_UNDERLINE;
    }
    if modifier.contains(Modifier::SLOW_BLINK) || modifier.contains(Modifier::RAPID_BLINK) {
        flags |= SharedCell::FLAG_BLINK;
    }
    if modifier.contains(Modifier::REVERSED) {
        flags |= SharedCell::FLAG_REVERSE;
    }
    if modifier.contains(Modifier::HIDDEN) {
        flags |= SharedCell::FLAG_HIDDEN;
    }
    if modifier.contains(Modifier::CROSSED_OUT) {
        flags |= SharedCell::FLAG_STRIKETHROUGH;
    }

    flags
}

/// Convert Scarab flags to Fusabi Modifier.
pub fn flags_to_modifier(flags: u8) -> Modifier {
    let mut modifier = Modifier::empty();

    if flags & SharedCell::FLAG_BOLD != 0 {
        modifier = modifier.insert(Modifier::BOLD);
    }
    if flags & SharedCell::FLAG_DIM != 0 {
        modifier = modifier.insert(Modifier::DIM);
    }
    if flags & SharedCell::FLAG_ITALIC != 0 {
        modifier = modifier.insert(Modifier::ITALIC);
    }
    if flags & SharedCell::FLAG_UNDERLINE != 0 {
        modifier = modifier.insert(Modifier::UNDERLINED);
    }
    if flags & SharedCell::FLAG_BLINK != 0 {
        modifier = modifier.insert(Modifier::SLOW_BLINK);
    }
    if flags & SharedCell::FLAG_REVERSE != 0 {
        modifier = modifier.insert(Modifier::REVERSED);
    }
    if flags & SharedCell::FLAG_HIDDEN != 0 {
        modifier = modifier.insert(Modifier::HIDDEN);
    }
    if flags & SharedCell::FLAG_STRIKETHROUGH != 0 {
        modifier = modifier.insert(Modifier::CROSSED_OUT);
    }

    modifier
}

/// Convert a Fusabi TUI Cell to a Scarab SharedCell.
pub fn tui_cell_to_shared(cell: &TuiCell) -> SharedCell {
    // Extract the first character (or use space if empty)
    let char_codepoint = cell
        .symbol
        .chars()
        .next()
        .unwrap_or(' ') as u32;

    let fg = color_to_u32(cell.fg);
    let bg = color_to_u32(cell.bg);
    let flags = modifier_to_flags(cell.modifier);

    SharedCell {
        char_codepoint,
        fg,
        bg,
        flags,
        _padding: [0; 3],
    }
}

/// Convert a Scarab SharedCell to a Fusabi TUI Cell.
pub fn shared_to_tui_cell(cell: &SharedCell) -> TuiCell {
    // Convert codepoint to string
    let symbol = char::from_u32(cell.char_codepoint)
        .unwrap_or(' ')
        .to_string();

    let fg = u32_to_color(cell.fg);
    let bg = u32_to_color(cell.bg);
    let modifier = flags_to_modifier(cell.flags);

    TuiCell {
        symbol,
        fg,
        bg,
        modifier,
    }
}

/// Implement From trait for convenience (wraps the public function)
impl From<&TuiCell> for SharedCell {
    fn from(cell: &TuiCell) -> Self {
        tui_cell_to_shared(cell)
    }
}

/// Implement From trait for convenience (wraps the public function)
impl From<&SharedCell> for TuiCell {
    fn from(cell: &SharedCell) -> Self {
        shared_to_tui_cell(cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_u32() {
        let red = color_to_u32(Color::Red);
        assert_eq!(red, 0xFFFF0000);

        let rgb = color_to_u32(Color::Rgb(128, 64, 32));
        assert_eq!(rgb, 0xFF804020);
    }

    #[test]
    fn test_u32_to_color() {
        let color = u32_to_color(0xFFFF0000);
        if let Color::Rgb(r, g, b) = color {
            assert_eq!(r, 255);
            assert_eq!(g, 0);
            assert_eq!(b, 0);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_modifier_to_flags() {
        let modifier = Modifier::BOLD | Modifier::ITALIC;
        let flags = modifier_to_flags(modifier);
        assert!(flags & SharedCell::FLAG_BOLD != 0);
        assert!(flags & SharedCell::FLAG_ITALIC != 0);
        assert!(flags & SharedCell::FLAG_UNDERLINE == 0);
    }

    #[test]
    fn test_flags_to_modifier() {
        let flags = SharedCell::FLAG_BOLD | SharedCell::FLAG_UNDERLINE;
        let modifier = flags_to_modifier(flags);
        assert!(modifier.contains(Modifier::BOLD));
        assert!(modifier.contains(Modifier::UNDERLINED));
        assert!(!modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_tui_cell_to_shared_cell() {
        let tui_cell = TuiCell {
            symbol: "A".to_string(),
            fg: Color::Red,
            bg: Color::Black,
            modifier: Modifier::BOLD,
        };

        let shared_cell = SharedCell::from(&tui_cell);
        assert_eq!(shared_cell.char_codepoint, 'A' as u32);
        assert_eq!(shared_cell.fg, 0xFFFF0000);
        assert_eq!(shared_cell.bg, 0xFF000000);
        assert!(shared_cell.flags & SharedCell::FLAG_BOLD != 0);
    }

    #[test]
    fn test_shared_cell_to_tui_cell() {
        let shared_cell = SharedCell {
            char_codepoint: 'B' as u32,
            fg: 0xFF00FF00,
            bg: 0xFFFFFFFF,
            flags: SharedCell::FLAG_ITALIC,
            _padding: [0; 3],
        };

        let tui_cell = TuiCell::from(&shared_cell);
        assert_eq!(tui_cell.symbol, "B");
        assert!(tui_cell.modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = TuiCell {
            symbol: "X".to_string(),
            fg: Color::Cyan,
            bg: Color::DarkGray,
            modifier: Modifier::BOLD | Modifier::UNDERLINED,
        };

        let shared = SharedCell::from(&original);
        let converted = TuiCell::from(&shared);

        assert_eq!(converted.symbol, "X");
        assert!(converted.modifier.contains(Modifier::BOLD));
        assert!(converted.modifier.contains(Modifier::UNDERLINED));
    }
}
