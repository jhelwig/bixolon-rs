# Task 4: Character Formatting Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/character.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Step 1: Add module declaration

Update `src/command/mod.rs`:

```rust
pub mod character;
```

---

## Step 2: Write failing tests for character formatting

Create `src/command/character.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn set_emphasized_on() {
        let cmd = SetEmphasized(true);
        assert_eq!(cmd.encode(), vec![0x1B, b'E', 1]);
    }

    #[test]
    fn set_emphasized_off() {
        let cmd = SetEmphasized(false);
        assert_eq!(cmd.encode(), vec![0x1B, b'E', 0]);
    }

    #[test]
    fn underline_off() {
        let cmd = SetUnderline(UnderlineThickness::Off);
        assert_eq!(cmd.encode(), vec![0x1B, b'-', 0]);
    }

    #[test]
    fn underline_one_dot() {
        let cmd = SetUnderline(UnderlineThickness::OneDot);
        assert_eq!(cmd.encode(), vec![0x1B, b'-', 1]);
    }

    #[test]
    fn character_size_double() {
        let cmd = SetCharacterSize { width: 2, height: 2 };
        // width=2 -> bits 4-6 = 0x10, height=2 -> bits 0-2 = 0x01
        assert_eq!(cmd.encode(), vec![0x1D, b'!', 0x11]);
    }

    #[test]
    fn justification_center() {
        let cmd = SetJustification(Justification::Center);
        assert_eq!(cmd.encode(), vec![0x1B, b'a', 1]);
    }
}
```

---

## Step 3: Run tests to verify they fail

```bash
cargo test -p bixolon command::character
```

Expected: Compilation error - types not defined

---

## Step 4: Implement character formatting commands

Update `src/command/character.rs`:

```rust
//! Character formatting commands.
//!
//! Commands for text styling: emphasis, underline, size, font, rotation, etc.

use crate::error::ValidationError;
use super::{Command, ESC, GS};

/// Turn emphasized (bold) mode on or off.
///
/// When enabled, characters are printed with increased weight.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `ESC E n` (0x1B 0x45 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetEmphasized(pub bool);

impl Command for SetEmphasized {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'E', if self.0 { 1 } else { 0 }]
    }
}

/// Underline thickness options.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnderlineThickness {
    /// Underline disabled.
    #[default]
    Off = 0,
    /// 1-dot thick underline.
    OneDot = 1,
    /// 2-dot thick underline.
    TwoDot = 2,
}

/// Turn underline mode on or off.
///
/// When enabled, characters are printed with an underline.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `ESC - n` (0x1B 0x2D n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetUnderline(pub UnderlineThickness);

impl Command for SetUnderline {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'-', self.0 as u8]
    }
}

/// Turn double-strike mode on or off.
///
/// When enabled, characters are printed twice for darker output.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `ESC G n` (0x1B 0x47 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetDoubleStrike(pub bool);

impl Command for SetDoubleStrike {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'G', if self.0 { 1 } else { 0 }]
    }
}

/// Character font selection.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Font {
    /// Font A: 12×24 dots.
    #[default]
    A = 0,
    /// Font B: 9×17 dots.
    B = 1,
}

/// Select character font.
///
/// ESC/POS: `ESC M n` (0x1B 0x4D n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectFont(pub Font);

impl Command for SelectFont {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'M', self.0 as u8]
    }
}

/// Set character size with independent width and height scaling.
///
/// Both width and height can be scaled 1-8 times.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `GS ! n` (0x1D 0x21 n)
///
/// # Encoding
///
/// The parameter byte encodes both scales:
/// - Bits 0-2: height scale (0=1x, 1=2x, ..., 7=8x)
/// - Bits 4-6: width scale (0=1x, 1=2x, ..., 7=8x)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetCharacterSize {
    /// Horizontal scaling factor (1-8).
    pub width: u8,
    /// Vertical scaling factor (1-8).
    pub height: u8,
}

impl SetCharacterSize {
    /// Create with validated scaling factors.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidScaleFactor`] if width or height is not 1-8.
    pub fn new(width: u8, height: u8) -> Result<Self, ValidationError> {
        if !(1..=8).contains(&width) || !(1..=8).contains(&height) {
            return Err(ValidationError::InvalidScaleFactor { width, height });
        }
        Ok(Self { width, height })
    }

    /// Standard size (1×1).
    pub const fn standard() -> Self {
        Self { width: 1, height: 1 }
    }

    /// Double width and height (2×2).
    pub const fn double() -> Self {
        Self { width: 2, height: 2 }
    }

    /// Double width only (2×1).
    pub const fn double_width() -> Self {
        Self { width: 2, height: 1 }
    }

    /// Double height only (1×2).
    pub const fn double_height() -> Self {
        Self { width: 1, height: 2 }
    }
}

impl Default for SetCharacterSize {
    fn default() -> Self {
        Self::standard()
    }
}

impl Command for SetCharacterSize {
    fn encode(&self) -> Vec<u8> {
        // Clamp to valid range and convert to 0-indexed
        let w = (self.width.clamp(1, 8) - 1) & 0x07;
        let h = (self.height.clamp(1, 8) - 1) & 0x07;
        let n = (w << 4) | h;
        vec![GS, b'!', n]
    }
}

/// Text justification options.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justification {
    /// Left justification (default).
    #[default]
    Left = 0,
    /// Center justification.
    Center = 1,
    /// Right justification.
    Right = 2,
}

/// Set text justification.
///
/// Affects all following text until changed. Only effective at the
/// beginning of a line.
///
/// ESC/POS: `ESC a n` (0x1B 0x61 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetJustification(pub Justification);

impl Command for SetJustification {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'a', self.0 as u8]
    }
}

/// Turn upside-down printing mode on or off.
///
/// When enabled, characters are rotated 180 degrees.
/// Must be sent BEFORE the text it affects, at the beginning of a line.
///
/// ESC/POS: `ESC { n` (0x1B 0x7B n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetUpsideDown(pub bool);

impl Command for SetUpsideDown {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'{', if self.0 { 1 } else { 0 }]
    }
}

/// Rotation mode options.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationMode {
    /// No rotation (default).
    #[default]
    Off = 0,
    /// 90° clockwise rotation.
    Clockwise90 = 1,
}

/// Turn 90° clockwise rotation mode on or off.
///
/// When enabled, characters are rotated 90° clockwise.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `ESC V n` (0x1B 0x56 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetRotation(pub RotationMode);

impl Command for SetRotation {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'V', self.0 as u8]
    }
}

/// Turn white/black reverse printing mode on or off.
///
/// When enabled, characters are printed white on black background.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `GS B n` (0x1D 0x42 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetReverse(pub bool);

impl Command for SetReverse {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'B', if self.0 { 1 } else { 0 }]
    }
}

/// Turn smoothing mode on or off.
///
/// When enabled, character edges are smoothed when enlarged.
/// Must be sent BEFORE the text it affects.
///
/// ESC/POS: `GS b n` (0x1D 0x62 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetSmoothing(pub bool);

impl Command for SetSmoothing {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'b', if self.0 { 1 } else { 0 }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_emphasized_on() {
        let cmd = SetEmphasized(true);
        assert_eq!(cmd.encode(), vec![0x1B, b'E', 1]);
    }

    #[test]
    fn set_emphasized_off() {
        let cmd = SetEmphasized(false);
        assert_eq!(cmd.encode(), vec![0x1B, b'E', 0]);
    }

    #[test]
    fn underline_off() {
        let cmd = SetUnderline(UnderlineThickness::Off);
        assert_eq!(cmd.encode(), vec![0x1B, b'-', 0]);
    }

    #[test]
    fn underline_one_dot() {
        let cmd = SetUnderline(UnderlineThickness::OneDot);
        assert_eq!(cmd.encode(), vec![0x1B, b'-', 1]);
    }

    #[test]
    fn underline_two_dot() {
        let cmd = SetUnderline(UnderlineThickness::TwoDot);
        assert_eq!(cmd.encode(), vec![0x1B, b'-', 2]);
    }

    #[test]
    fn double_strike_on() {
        let cmd = SetDoubleStrike(true);
        assert_eq!(cmd.encode(), vec![0x1B, b'G', 1]);
    }

    #[test]
    fn select_font_a() {
        let cmd = SelectFont(Font::A);
        assert_eq!(cmd.encode(), vec![0x1B, b'M', 0]);
    }

    #[test]
    fn select_font_b() {
        let cmd = SelectFont(Font::B);
        assert_eq!(cmd.encode(), vec![0x1B, b'M', 1]);
    }

    #[test]
    fn character_size_standard() {
        let cmd = SetCharacterSize::standard();
        assert_eq!(cmd.encode(), vec![0x1D, b'!', 0x00]);
    }

    #[test]
    fn character_size_double() {
        let cmd = SetCharacterSize::double();
        // width=2 -> (2-1)<<4 = 0x10, height=2 -> (2-1) = 0x01
        assert_eq!(cmd.encode(), vec![0x1D, b'!', 0x11]);
    }

    #[test]
    fn character_size_max() {
        let cmd = SetCharacterSize { width: 8, height: 8 };
        // width=8 -> (8-1)<<4 = 0x70, height=8 -> (8-1) = 0x07
        assert_eq!(cmd.encode(), vec![0x1D, b'!', 0x77]);
    }

    #[test]
    fn character_size_validation_rejects_zero() {
        let result = SetCharacterSize::new(0, 1);
        assert!(matches!(result, Err(ValidationError::InvalidScaleFactor { .. })));
    }

    #[test]
    fn character_size_validation_rejects_nine() {
        let result = SetCharacterSize::new(1, 9);
        assert!(matches!(result, Err(ValidationError::InvalidScaleFactor { .. })));
    }

    #[test]
    fn justification_left() {
        let cmd = SetJustification(Justification::Left);
        assert_eq!(cmd.encode(), vec![0x1B, b'a', 0]);
    }

    #[test]
    fn justification_center() {
        let cmd = SetJustification(Justification::Center);
        assert_eq!(cmd.encode(), vec![0x1B, b'a', 1]);
    }

    #[test]
    fn justification_right() {
        let cmd = SetJustification(Justification::Right);
        assert_eq!(cmd.encode(), vec![0x1B, b'a', 2]);
    }

    #[test]
    fn upside_down_on() {
        let cmd = SetUpsideDown(true);
        assert_eq!(cmd.encode(), vec![0x1B, b'{', 1]);
    }

    #[test]
    fn rotation_clockwise() {
        let cmd = SetRotation(RotationMode::Clockwise90);
        assert_eq!(cmd.encode(), vec![0x1B, b'V', 1]);
    }

    #[test]
    fn reverse_on() {
        let cmd = SetReverse(true);
        assert_eq!(cmd.encode(), vec![0x1D, b'B', 1]);
    }

    #[test]
    fn smoothing_on() {
        let cmd = SetSmoothing(true);
        assert_eq!(cmd.encode(), vec![0x1D, b'b', 1]);
    }
}
```

---

## Step 5: Run tests to verify they pass

```bash
cargo test -p bixolon command::character
```

Expected: All tests pass

---

## Step 6: Commit

```bash
git add src/command/character.rs src/command/mod.rs
git commit -m "feat: add character formatting commands (emphasis, underline, size, font, rotation)"
```
