//! Border types and border flags for widgets.

use bitflags::bitflags;

bitflags! {
    /// Bitflags for specifying which borders to draw.
    ///
    /// Can be combined using bitwise operations to draw multiple borders.
    ///
    /// # Examples
    ///
    /// ```
    /// use fusabi_tui_widgets::borders::Borders;
    ///
    /// // Draw all borders
    /// let borders = Borders::ALL;
    ///
    /// // Draw only top and bottom
    /// let borders = Borders::TOP | Borders::BOTTOM;
    ///
    /// // Draw left, right, and bottom
    /// let borders = Borders::LEFT | Borders::RIGHT | Borders::BOTTOM;
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Borders: u8 {
        /// No borders
        const NONE = 0b0000;
        /// Top border
        const TOP = 0b0001;
        /// Right border
        const RIGHT = 0b0010;
        /// Bottom border
        const BOTTOM = 0b0100;
        /// Left border
        const LEFT = 0b1000;
        /// All borders (top, right, bottom, left)
        const ALL = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

impl Default for Borders {
    fn default() -> Self {
        Self::NONE
    }
}

/// The type of border characters to use when drawing.
///
/// Different border types use different Unicode characters for drawing the border lines
/// and corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorderType {
    /// Plain borders using simple box-drawing characters (┌─┐│└─┘)
    Plain,
    /// Rounded borders using rounded corners (╭─╮│╰─╯)
    Rounded,
    /// Double-line borders using double box-drawing characters (╔═╗║╚═╝)
    Double,
    /// Thick borders using heavy box-drawing characters (┏━┓┃┗━┛)
    Thick,
}

impl Default for BorderType {
    fn default() -> Self {
        Self::Plain
    }
}

impl BorderType {
    /// Returns the Unicode characters for this border type.
    ///
    /// Returns a tuple of (horizontal, vertical, top_left, top_right, bottom_left, bottom_right).
    pub(crate) fn line_symbols(self) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
        use fusabi_tui_core::symbols::line;

        match self {
            BorderType::Plain => (
                line::HORIZONTAL,
                line::VERTICAL,
                line::TOP_LEFT,
                line::TOP_RIGHT,
                line::BOTTOM_LEFT,
                line::BOTTOM_RIGHT,
            ),
            BorderType::Rounded => (
                line::HORIZONTAL,
                line::VERTICAL,
                line::ROUNDED_TOP_LEFT,
                line::ROUNDED_TOP_RIGHT,
                line::ROUNDED_BOTTOM_LEFT,
                line::ROUNDED_BOTTOM_RIGHT,
            ),
            BorderType::Double => (
                line::DOUBLE_HORIZONTAL,
                line::DOUBLE_VERTICAL,
                line::DOUBLE_TOP_LEFT,
                line::DOUBLE_TOP_RIGHT,
                line::DOUBLE_BOTTOM_LEFT,
                line::DOUBLE_BOTTOM_RIGHT,
            ),
            BorderType::Thick => (
                line::THICK_HORIZONTAL,
                line::THICK_VERTICAL,
                line::THICK_TOP_LEFT,
                line::THICK_TOP_RIGHT,
                line::THICK_BOTTOM_LEFT,
                line::THICK_BOTTOM_RIGHT,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borders_none() {
        let borders = Borders::NONE;
        assert!(!borders.contains(Borders::TOP));
        assert!(!borders.contains(Borders::RIGHT));
        assert!(!borders.contains(Borders::BOTTOM));
        assert!(!borders.contains(Borders::LEFT));
    }

    #[test]
    fn test_borders_all() {
        let borders = Borders::ALL;
        assert!(borders.contains(Borders::TOP));
        assert!(borders.contains(Borders::RIGHT));
        assert!(borders.contains(Borders::BOTTOM));
        assert!(borders.contains(Borders::LEFT));
    }

    #[test]
    fn test_borders_combination() {
        let borders = Borders::TOP | Borders::BOTTOM;
        assert!(borders.contains(Borders::TOP));
        assert!(!borders.contains(Borders::RIGHT));
        assert!(borders.contains(Borders::BOTTOM));
        assert!(!borders.contains(Borders::LEFT));
    }

    #[test]
    fn test_borders_default() {
        assert_eq!(Borders::default(), Borders::NONE);
    }

    #[test]
    fn test_border_type_default() {
        assert_eq!(BorderType::default(), BorderType::Plain);
    }

    #[test]
    fn test_border_type_plain_symbols() {
        let (h, v, tl, tr, bl, br) = BorderType::Plain.line_symbols();
        assert_eq!(h, "─");
        assert_eq!(v, "│");
        assert_eq!(tl, "┌");
        assert_eq!(tr, "┐");
        assert_eq!(bl, "└");
        assert_eq!(br, "┘");
    }

    #[test]
    fn test_border_type_rounded_symbols() {
        let (h, v, tl, tr, bl, br) = BorderType::Rounded.line_symbols();
        assert_eq!(h, "─");
        assert_eq!(v, "│");
        assert_eq!(tl, "╭");
        assert_eq!(tr, "╮");
        assert_eq!(bl, "╰");
        assert_eq!(br, "╯");
    }

    #[test]
    fn test_border_type_double_symbols() {
        let (h, v, tl, tr, bl, br) = BorderType::Double.line_symbols();
        assert_eq!(h, "═");
        assert_eq!(v, "║");
        assert_eq!(tl, "╔");
        assert_eq!(tr, "╗");
        assert_eq!(bl, "╚");
        assert_eq!(br, "╝");
    }

    #[test]
    fn test_border_type_thick_symbols() {
        let (h, v, tl, tr, bl, br) = BorderType::Thick.line_symbols();
        assert_eq!(h, "━");
        assert_eq!(v, "┃");
        assert_eq!(tl, "┏");
        assert_eq!(tr, "┓");
        assert_eq!(bl, "┗");
        assert_eq!(br, "┛");
    }
}
