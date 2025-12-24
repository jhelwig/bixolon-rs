# Task 11: Image Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/image.rs`
- Modify: `src/command/mod.rs`

---

## Commands to Implement

| Command | ESC/POS | Description |
|---------|---------|-------------|
| `SelectBitImageMode` | `ESC * m nL nH d...` | Select bit image mode and print |
| `DefineDownloadedImage` | `GS * x y d...` | Define downloaded bit image |
| `PrintDownloadedImage` | `GS / m` | Print downloaded bit image |
| `PrintRasterImage` | `GS v 0 m xL xH yL yH d...` | Print raster bit image |

---

## Implementation Notes

- Bit image modes: 8-dot single/double density, 24-dot single/double density
- Raster images: Support different density modes
- NV images: Document 10/day write limit in API docs

```rust
//! Image commands.

use super::{Command, ESC, GS};

/// Bit image mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitImageMode {
    /// 8-dot single density (60 dpi V, 90 dpi H).
    SingleDensity8 = 0,
    /// 8-dot double density (60 dpi V, 180 dpi H).
    DoubleDensity8 = 1,
    /// 24-dot single density (180 dpi V, 90 dpi H).
    SingleDensity24 = 32,
    /// 24-dot double density (180 dpi V, 180 dpi H).
    #[default]
    DoubleDensity24 = 33,
}

/// Print raster bit image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintRasterImage {
    /// Image mode/density.
    pub mode: RasterImageMode,
    /// Width in bytes (8 pixels per byte).
    pub width_bytes: u16,
    /// Height in dots.
    pub height_dots: u16,
    /// Image data (1 bit per pixel, MSB first).
    pub data: Vec<u8>,
}

/// Raster image mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RasterImageMode {
    /// Normal (180 dpi).
    #[default]
    Normal = 0,
    /// Double width (90 dpi H).
    DoubleWidth = 1,
    /// Double height (90 dpi V).
    DoubleHeight = 2,
    /// Quadruple (90 dpi both).
    Quadruple = 3,
}

impl Command for PrintRasterImage {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![
            GS, b'v', b'0', self.mode as u8,
            (self.width_bytes & 0xFF) as u8,
            ((self.width_bytes >> 8) & 0xFF) as u8,
            (self.height_dots & 0xFF) as u8,
            ((self.height_dots >> 8) & 0xFF) as u8,
        ];
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

// Additional image commands follow same pattern...
```

---

## Commit

```bash
git add src/command/image.rs src/command/mod.rs
git commit -m "feat: add image commands"
```
