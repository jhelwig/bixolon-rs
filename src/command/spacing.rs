//! Spacing and positioning commands.

use super::{Command, ESC, GS};

/// Set default line spacing (~4.23mm / 1/6 inch).
///
/// ESC/POS: `ESC 2` (0x1B 0x32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetDefaultLineSpacing;

impl Command for SetDefaultLineSpacing {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'2']
    }
}

/// Set line spacing in vertical motion units.
///
/// ESC/POS: `ESC 3 n` (0x1B 0x33 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetLineSpacing(pub u8);

impl Command for SetLineSpacing {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'3', self.0]
    }
}

/// Set right-side character spacing.
///
/// ESC/POS: `ESC SP n` (0x1B 0x20 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetRightSpacing(pub u8);

impl Command for SetRightSpacing {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b' ', self.0]
    }
}

/// Set horizontal tab positions.
///
/// ESC/POS: `ESC D n1...nk NUL` (0x1B 0x44 n1...nk 0x00)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetHorizontalTabs {
    /// Tab positions (must be in ascending order, max 32 positions).
    pub positions: Vec<u8>,
}

impl SetHorizontalTabs {
    /// Clear all tab positions.
    pub fn clear() -> Self {
        Self {
            positions: vec![],
        }
    }
}

impl Command for SetHorizontalTabs {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![ESC, b'D'];
        bytes.extend_from_slice(&self.positions);
        bytes.push(0x00); // NUL terminator
        bytes
    }
}

/// Set absolute print position.
///
/// ESC/POS: `ESC $ nL nH` (0x1B 0x24 nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetAbsolutePosition(pub u16);

impl Command for SetAbsolutePosition {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.0 & 0xFF) as u8;
        let nh = ((self.0 >> 8) & 0xFF) as u8;
        vec![ESC, b'$', nl, nh]
    }
}

/// Set relative print position.
///
/// ESC/POS: `ESC \ nL nH` (0x1B 0x5C nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetRelativePosition(pub i16);

impl Command for SetRelativePosition {
    fn encode(&self) -> Vec<u8> {
        let value = self.0 as u16;
        let nl = (value & 0xFF) as u8;
        let nh = ((value >> 8) & 0xFF) as u8;
        vec![ESC, b'\\', nl, nh]
    }
}

/// Set left margin.
///
/// ESC/POS: `GS L nL nH` (0x1D 0x4C nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetLeftMargin(pub u16);

impl Command for SetLeftMargin {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.0 & 0xFF) as u8;
        let nh = ((self.0 >> 8) & 0xFF) as u8;
        vec![GS, b'L', nl, nh]
    }
}

/// Set printing area width.
///
/// ESC/POS: `GS W nL nH` (0x1D 0x57 nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetPrintingWidth(pub u16);

impl Command for SetPrintingWidth {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.0 & 0xFF) as u8;
        let nh = ((self.0 >> 8) & 0xFF) as u8;
        vec![GS, b'W', nl, nh]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_default_line_spacing_encodes() {
        let cmd = SetDefaultLineSpacing;
        assert_eq!(cmd.encode(), vec![0x1B, b'2']);
    }

    #[test]
    fn set_line_spacing_encodes() {
        let cmd = SetLineSpacing(60);
        assert_eq!(cmd.encode(), vec![0x1B, b'3', 60]);
    }

    #[test]
    fn set_right_spacing_encodes() {
        let cmd = SetRightSpacing(5);
        assert_eq!(cmd.encode(), vec![0x1B, b' ', 5]);
    }

    #[test]
    fn set_horizontal_tabs_encodes() {
        let cmd = SetHorizontalTabs {
            positions: vec![8, 16, 24],
        };
        assert_eq!(cmd.encode(), vec![0x1B, b'D', 8, 16, 24, 0]);
    }

    #[test]
    fn set_horizontal_tabs_clear() {
        let cmd = SetHorizontalTabs::clear();
        assert_eq!(cmd.encode(), vec![0x1B, b'D', 0]);
    }

    #[test]
    fn set_absolute_position_encodes() {
        let cmd = SetAbsolutePosition(256);
        assert_eq!(cmd.encode(), vec![0x1B, b'$', 0, 1]);
    }

    #[test]
    fn set_relative_position_positive() {
        let cmd = SetRelativePosition(100);
        assert_eq!(cmd.encode(), vec![0x1B, b'\\', 100, 0]);
    }

    #[test]
    fn set_relative_position_negative() {
        let cmd = SetRelativePosition(-100);
        // -100 as u16 = 0xFF9C
        assert_eq!(cmd.encode(), vec![0x1B, b'\\', 0x9C, 0xFF]);
    }

    #[test]
    fn set_left_margin_encodes() {
        let cmd = SetLeftMargin(50);
        assert_eq!(cmd.encode(), vec![0x1D, b'L', 50, 0]);
    }

    #[test]
    fn set_printing_width_encodes() {
        let cmd = SetPrintingWidth(512);
        assert_eq!(cmd.encode(), vec![0x1D, b'W', 0, 2]);
    }
}
