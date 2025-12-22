# Task 6: Spacing and Positioning Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/spacing.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Commands to Implement

| Command | ESC/POS | Description |
|---------|---------|-------------|
| `SetRightSpacing` | `ESC SP n` | Set right-side character spacing |
| `SetDefaultLineSpacing` | `ESC 2` | Set default line spacing (~4.23mm) |
| `SetLineSpacing` | `ESC 3 n` | Set line spacing in dots |
| `SetHorizontalTabs` | `ESC D n1..nk NUL` | Set tab positions |
| `SetAbsolutePosition` | `ESC $ nL nH` | Set absolute horizontal position |
| `SetRelativePosition` | `ESC \ nL nH` | Set relative horizontal position |
| `SetLeftMargin` | `GS L nL nH` | Set left margin |
| `SetPrintingWidth` | `GS W nL nH` | Set printing area width |

---

## Step 1: Add module declaration

Update `src/command/mod.rs`:

```rust
pub mod spacing;
```

---

## Step 2: Write failing tests

Create `src/command/spacing.rs` with tests for each command.

---

## Step 3: Implement spacing commands

```rust
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
        Self { positions: vec![] }
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

// Add comprehensive tests for all commands
```

---

## Step 4-6: Run tests, verify pass, commit

```bash
cargo test -p bixolon command::spacing
git add src/command/spacing.rs src/command/mod.rs
git commit -m "feat: add spacing and positioning commands"
```
