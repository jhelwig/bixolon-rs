//! Page mode commands.
//!
//! Page mode buffers all output and prints when FormFeed is sent.

use super::{Command, ESC, GS};

/// Enter page mode.
///
/// Switches from standard mode to page mode. In page mode, all print
/// data is buffered in the print area until FormFeed is sent.
///
/// ESC/POS: `ESC L` (0x1B 0x4C)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnterPageMode;

impl Command for EnterPageMode {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'L']
    }
}

/// Exit page mode and return to standard mode.
///
/// Clears all data in the page buffer and returns to standard mode.
///
/// ESC/POS: `ESC S` (0x1B 0x53)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExitPageMode;

impl Command for ExitPageMode {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'S']
    }
}

/// Print direction in page mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintDirection {
    /// Left to right, starting upper-left.
    #[default]
    LeftToRight = 0,
    /// Bottom to top, starting lower-left.
    BottomToTop = 1,
    /// Right to left, starting lower-right.
    RightToLeft = 2,
    /// Top to bottom, starting upper-right.
    TopToBottom = 3,
}

/// Set print direction in page mode.
///
/// ESC/POS: `ESC T n` (0x1B 0x54 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetPrintDirection(pub PrintDirection);

impl Command for SetPrintDirection {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'T', self.0 as u8]
    }
}

/// Print area definition for page mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintArea {
    /// Horizontal start position.
    pub x: u16,
    /// Vertical start position.
    pub y: u16,
    /// Width of print area.
    pub width: u16,
    /// Height of print area.
    pub height: u16,
}

impl PrintArea {
    /// Default print area for 80mm paper.
    pub const fn default_80mm() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 512,
            height: 1662,
        }
    }

    /// Default print area for 58mm paper.
    pub const fn default_58mm() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 360,
            height: 1662,
        }
    }
}

/// Set printing area in page mode.
///
/// Defines the boundaries of the printable area.
///
/// ESC/POS: `ESC W xL xH yL yH dxL dxH dyL dyH`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetPrintArea(pub PrintArea);

impl Command for SetPrintArea {
    fn encode(&self) -> Vec<u8> {
        let area = &self.0;
        vec![
            ESC,
            b'W',
            (area.x & 0xFF) as u8,
            ((area.x >> 8) & 0xFF) as u8,
            (area.y & 0xFF) as u8,
            ((area.y >> 8) & 0xFF) as u8,
            (area.width & 0xFF) as u8,
            ((area.width >> 8) & 0xFF) as u8,
            (area.height & 0xFF) as u8,
            ((area.height >> 8) & 0xFF) as u8,
        ]
    }
}

/// Set absolute horizontal print position in page mode.
///
/// Sets the horizontal position relative to the left edge of the print area.
/// Position is in motion units (typically 1/180 inch).
///
/// ESC/POS: `ESC $ nL nH` (0x1B 0x24 nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetHorizontalPosition(pub u16);

impl Command for SetHorizontalPosition {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.0 & 0xFF) as u8;
        let nh = ((self.0 >> 8) & 0xFF) as u8;
        vec![ESC, b'$', nl, nh]
    }
}

/// Set absolute vertical print position in page mode.
///
/// ESC/POS: `GS $ nL nH` (0x1D 0x24 nL nH)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetVerticalPosition(pub u16);

impl Command for SetVerticalPosition {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.0 & 0xFF) as u8;
        let nh = ((self.0 >> 8) & 0xFF) as u8;
        vec![GS, b'$', nl, nh]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_page_mode_encodes() {
        assert_eq!(EnterPageMode.encode(), vec![0x1B, b'L']);
    }

    #[test]
    fn exit_page_mode_encodes() {
        assert_eq!(ExitPageMode.encode(), vec![0x1B, b'S']);
    }

    #[test]
    fn print_direction_values() {
        assert_eq!(PrintDirection::LeftToRight as u8, 0);
        assert_eq!(PrintDirection::BottomToTop as u8, 1);
        assert_eq!(PrintDirection::RightToLeft as u8, 2);
        assert_eq!(PrintDirection::TopToBottom as u8, 3);
    }

    #[test]
    fn set_print_direction_encodes() {
        let cmd = SetPrintDirection(PrintDirection::BottomToTop);
        assert_eq!(cmd.encode(), vec![0x1B, b'T', 1]);
    }

    #[test]
    fn set_print_area_encodes() {
        let area = PrintArea {
            x: 0,
            y: 0,
            width: 512,
            height: 400,
        };
        let cmd = SetPrintArea(area);
        let encoded = cmd.encode();
        assert_eq!(encoded.len(), 10);
        assert_eq!(encoded[0..2], [0x1B, b'W']);
    }

    #[test]
    fn set_vertical_position_encodes() {
        let cmd = SetVerticalPosition(256);
        assert_eq!(cmd.encode(), vec![0x1D, b'$', 0, 1]);
    }

    #[test]
    fn set_horizontal_position_encodes() {
        let cmd = SetHorizontalPosition(256);
        assert_eq!(cmd.encode(), vec![0x1B, b'$', 0, 1]);
    }
}
