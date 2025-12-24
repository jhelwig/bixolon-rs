# Task 8: Paper and Cutting Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/paper.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Commands to Implement

| Command | ESC/POS | Description |
|---------|---------|-------------|
| `FeedPaper` | `ESC J n` | Print buffer and feed n dots |
| `FeedLines` | `ESC d n` | Print buffer and feed n lines |
| `CutPaper` | `GS V m [n]` | Select cut mode and cut |

---

## Implementation

```rust
//! Paper feed and cutting commands.

use super::{Command, ESC, GS};

/// Print buffer and feed paper by specified dots.
///
/// ESC/POS: `ESC J n` (0x1B 0x4A n)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedPaper(pub u8);

impl Command for FeedPaper {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'J', self.0]
    }
}

/// Print buffer and feed paper by specified lines.
///
/// ESC/POS: `ESC d n` (0x1B 0x64 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedLines(pub u8);

impl Command for FeedLines {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'd', self.0]
    }
}

/// Cut mode selection.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CutMode {
    /// Full cut (or partial, depending on hardware setting).
    #[default]
    Full = 0,
    /// Partial cut (one point left uncut).
    Partial = 1,
    /// Feed and full cut.
    FeedAndFull = 65,
    /// Feed and partial cut.
    FeedAndPartial = 66,
}

/// Cut paper.
///
/// ESC/POS: `GS V m [n]` (0x1D 0x56 m [n])
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutPaper {
    /// Cut mode.
    pub mode: CutMode,
    /// Feed amount before cutting (only for FeedAndFull/FeedAndPartial).
    pub feed_lines: Option<u8>,
}

impl CutPaper {
    /// Full cut without feed.
    pub const fn full() -> Self {
        Self { mode: CutMode::Full, feed_lines: None }
    }

    /// Partial cut without feed.
    pub const fn partial() -> Self {
        Self { mode: CutMode::Partial, feed_lines: None }
    }

    /// Feed specified lines then full cut.
    pub const fn feed_and_full(lines: u8) -> Self {
        Self { mode: CutMode::FeedAndFull, feed_lines: Some(lines) }
    }

    /// Feed specified lines then partial cut.
    pub const fn feed_and_partial(lines: u8) -> Self {
        Self { mode: CutMode::FeedAndPartial, feed_lines: Some(lines) }
    }
}

impl Default for CutPaper {
    fn default() -> Self {
        Self::full()
    }
}

impl Command for CutPaper {
    fn encode(&self) -> Vec<u8> {
        match self.feed_lines {
            Some(n) => vec![GS, b'V', self.mode as u8, n],
            None => vec![GS, b'V', self.mode as u8],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_paper_encodes() {
        let cmd = FeedPaper(100);
        assert_eq!(cmd.encode(), vec![0x1B, b'J', 100]);
    }

    #[test]
    fn feed_lines_encodes() {
        let cmd = FeedLines(5);
        assert_eq!(cmd.encode(), vec![0x1B, b'd', 5]);
    }

    #[test]
    fn cut_full_encodes() {
        let cmd = CutPaper::full();
        assert_eq!(cmd.encode(), vec![0x1D, b'V', 0]);
    }

    #[test]
    fn cut_partial_encodes() {
        let cmd = CutPaper::partial();
        assert_eq!(cmd.encode(), vec![0x1D, b'V', 1]);
    }

    #[test]
    fn cut_feed_and_full_encodes() {
        let cmd = CutPaper::feed_and_full(3);
        assert_eq!(cmd.encode(), vec![0x1D, b'V', 65, 3]);
    }

    #[test]
    fn cut_feed_and_partial_encodes() {
        let cmd = CutPaper::feed_and_partial(5);
        assert_eq!(cmd.encode(), vec![0x1D, b'V', 66, 5]);
    }
}
```

---

## Commit

```bash
git add src/command/paper.rs src/command/mod.rs
git commit -m "feat: add paper feed and cutting commands"
```
