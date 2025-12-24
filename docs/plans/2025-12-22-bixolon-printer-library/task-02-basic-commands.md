# Task 2: Basic Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/basic.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Step 1: Add module declaration

Update `src/command/mod.rs`, add after constants:

```rust
pub mod basic;
```

---

## Step 2: Write failing tests for basic commands

Create `src/command/basic.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn line_feed_encodes_to_0x0a() {
        let cmd = LineFeed;
        assert_eq!(cmd.encode(), vec![0x0A]);
    }

    #[test]
    fn form_feed_encodes_to_0x0c() {
        let cmd = FormFeed;
        assert_eq!(cmd.encode(), vec![0x0C]);
    }

    #[test]
    fn carriage_return_encodes_to_0x0d() {
        let cmd = CarriageReturn;
        assert_eq!(cmd.encode(), vec![0x0D]);
    }

    #[test]
    fn horizontal_tab_encodes_to_0x09() {
        let cmd = HorizontalTab;
        assert_eq!(cmd.encode(), vec![0x09]);
    }

    #[test]
    fn cancel_encodes_to_0x18() {
        let cmd = Cancel;
        assert_eq!(cmd.encode(), vec![0x18]);
    }
}
```

---

## Step 3: Run tests to verify they fail

```bash
cargo test -p bixolon command::basic
```

Expected: Compilation error - structs not defined

---

## Step 4: Implement basic commands

Update `src/command/basic.rs`:

```rust
//! Basic line control commands.
//!
//! These are single-byte commands for fundamental printer operations.

use super::{Command, LF, FF, CR, HT, CAN};

/// Print buffer and feed one line.
///
/// Prints the data in the print buffer and feeds one line based on
/// the current line spacing.
///
/// ESC/POS: `LF` (0x0A)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineFeed;

impl Command for LineFeed {
    fn encode(&self) -> Vec<u8> {
        vec![LF]
    }
}

/// Print and return to standard mode (page mode only).
///
/// In page mode, prints all buffered data in the printing area and
/// returns to standard mode. In standard mode, this command is ignored.
///
/// ESC/POS: `FF` (0x0C)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormFeed;

impl Command for FormFeed {
    fn encode(&self) -> Vec<u8> {
        vec![FF]
    }
}

/// Print and carriage return.
///
/// When auto line feed is enabled (via DIP switch), functions as line feed.
/// When disabled, this command is ignored.
///
/// ESC/POS: `CR` (0x0D)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CarriageReturn;

impl Command for CarriageReturn {
    fn encode(&self) -> Vec<u8> {
        vec![CR]
    }
}

/// Move to next horizontal tab position.
///
/// Moves the print position to the next horizontal tab position.
/// Tab positions are set with `SetHorizontalTabs`.
///
/// ESC/POS: `HT` (0x09)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HorizontalTab;

impl Command for HorizontalTab {
    fn encode(&self) -> Vec<u8> {
        vec![HT]
    }
}

/// Cancel print data in page mode.
///
/// In page mode, deletes all print data in the current printable area.
/// In standard mode, this command is ignored.
///
/// ESC/POS: `CAN` (0x18)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cancel;

impl Command for Cancel {
    fn encode(&self) -> Vec<u8> {
        vec![CAN]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_feed_encodes_to_0x0a() {
        let cmd = LineFeed;
        assert_eq!(cmd.encode(), vec![0x0A]);
    }

    #[test]
    fn form_feed_encodes_to_0x0c() {
        let cmd = FormFeed;
        assert_eq!(cmd.encode(), vec![0x0C]);
    }

    #[test]
    fn carriage_return_encodes_to_0x0d() {
        let cmd = CarriageReturn;
        assert_eq!(cmd.encode(), vec![0x0D]);
    }

    #[test]
    fn horizontal_tab_encodes_to_0x09() {
        let cmd = HorizontalTab;
        assert_eq!(cmd.encode(), vec![0x09]);
    }

    #[test]
    fn cancel_encodes_to_0x18() {
        let cmd = Cancel;
        assert_eq!(cmd.encode(), vec![0x18]);
    }

    #[test]
    fn commands_implement_debug() {
        // Ensure Debug is implemented for all commands
        let _ = format!("{:?}", LineFeed);
        let _ = format!("{:?}", FormFeed);
        let _ = format!("{:?}", CarriageReturn);
        let _ = format!("{:?}", HorizontalTab);
        let _ = format!("{:?}", Cancel);
    }

    #[test]
    fn commands_are_copy() {
        // Ensure Copy is implemented
        let lf = LineFeed;
        let lf2 = lf;
        assert_eq!(lf.encode(), lf2.encode());
    }
}
```

---

## Step 5: Run tests to verify they pass

```bash
cargo test -p bixolon command::basic
```

Expected: All tests pass

---

## Step 6: Commit

```bash
git add src/command/basic.rs src/command/mod.rs
git commit -m "feat: add basic line control commands (LF, FF, CR, HT, CAN)"
```
