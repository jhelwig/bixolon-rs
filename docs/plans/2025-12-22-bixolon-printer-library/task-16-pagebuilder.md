# Task 16: PageBuilder

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/page/mod.rs`
- Modify: `src/lib.rs` (add module)

**Important:** PageBuilder is a command queue, not text-specific. It can hold any command, including styled text, barcodes, and images.

---

## Core Concepts

1. **Command queue**: Stores any `Command` implementor
2. **Page mode wrapping**: Automatically adds EnterPageMode/ExitPageMode
3. **Convenience methods**: Only for documented page mode operations

---

## Implementation

```rust
//! Page mode builder.
//!
//! [`PageBuilder`] accumulates commands for page mode printing.
//! In page mode, all output is buffered until FormFeed is sent.

use crate::command::{Command, basic::FormFeed};
use crate::command::page_mode::{
    EnterPageMode, ExitPageMode, PrintArea, PrintDirection,
    SetPrintArea, SetPrintDirection, SetVerticalPosition,
};
use crate::style::text::StyledNode;

/// Builder for page mode printing.
///
/// Accumulates commands to be printed in page mode. When rendered,
/// automatically wraps with EnterPageMode and FormFeed.
///
/// # Example
///
/// ```ignore
/// use bixolon::page::PageBuilder;
/// use bixolon::command::page_mode::PrintArea;
///
/// let page = PageBuilder::new()
///     .area(PrintArea::default_80mm())
///     .text("Header".bold())
///     .vertical_position(100)
///     .text("Body text")
///     .build();
///
/// printer.send_all(&page)?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct PageBuilder {
    /// Print area configuration.
    area: Option<PrintArea>,
    /// Print direction configuration.
    direction: Option<PrintDirection>,
    /// Queued commands.
    commands: Vec<QueuedCommand>,
}

/// A command in the page builder queue.
#[derive(Debug, Clone)]
enum QueuedCommand {
    /// Raw command bytes.
    Raw(Vec<u8>),
    /// Styled text node.
    StyledText(StyledNode),
}

impl PageBuilder {
    /// Create a new empty page builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the print area.
    pub fn area(mut self, area: PrintArea) -> Self {
        self.area = Some(area);
        self
    }

    /// Set the print direction.
    pub fn direction(mut self, direction: PrintDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Set absolute vertical position.
    pub fn vertical_position(mut self, position: u16) -> Self {
        self.commands.push(QueuedCommand::Raw(
            SetVerticalPosition(position).encode()
        ));
        self
    }

    /// Add a command to the queue.
    pub fn command(mut self, cmd: &impl Command) -> Self {
        self.commands.push(QueuedCommand::Raw(cmd.encode()));
        self
    }

    /// Add styled text.
    pub fn text(mut self, node: impl Into<StyledNode>) -> Self {
        self.commands.push(QueuedCommand::StyledText(node.into()));
        self
    }

    /// Add styled text followed by a line feed.
    pub fn text_line(mut self, node: impl Into<StyledNode>) -> Self {
        let node = node.into();
        self.commands.push(QueuedCommand::Raw(node.render_line()));
        self
    }

    /// Add raw bytes to the queue.
    pub fn raw(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.commands.push(QueuedCommand::Raw(bytes.into()));
        self
    }

    /// Build the complete page mode byte sequence.
    ///
    /// Returns bytes ready to send to the printer, including:
    /// 1. EnterPageMode command
    /// 2. Print area (if set)
    /// 3. Print direction (if set)
    /// 4. All queued commands
    /// 5. FormFeed to print the page
    pub fn build(&self) -> Vec<u8> {
        let mut output = Vec::new();

        // Enter page mode
        output.extend(EnterPageMode.encode());

        // Set print area if configured
        if let Some(area) = &self.area {
            output.extend(SetPrintArea(*area).encode());
        }

        // Set print direction if configured
        if let Some(direction) = &self.direction {
            output.extend(SetPrintDirection(*direction).encode());
        }

        // Render all queued commands
        for cmd in &self.commands {
            match cmd {
                QueuedCommand::Raw(bytes) => output.extend(bytes),
                QueuedCommand::StyledText(node) => output.extend(node.render()),
            }
        }

        // FormFeed to print the page
        output.extend(FormFeed.encode());

        output
    }

    /// Build and include ExitPageMode at the end.
    ///
    /// Use this when you want to return to standard mode after printing.
    pub fn build_and_exit(&self) -> Vec<u8> {
        let mut output = self.build();
        output.extend(ExitPageMode.encode());
        output
    }

    /// Check if the builder is empty (no commands queued).
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get the number of queued commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{ESC, GS};

    #[test]
    fn empty_page_has_enter_and_ff() {
        let page = PageBuilder::new().build();

        // Should start with ESC L (enter page mode)
        assert_eq!(&page[0..2], &[ESC, b'L']);

        // Should end with FF (form feed)
        assert_eq!(page.last(), Some(&0x0C));
    }

    #[test]
    fn page_with_area() {
        let page = PageBuilder::new()
            .area(PrintArea::default_80mm())
            .build();

        // Should contain ESC W (set print area)
        assert!(page.windows(2).any(|w| w == [ESC, b'W']));
    }

    #[test]
    fn page_with_direction() {
        let page = PageBuilder::new()
            .direction(PrintDirection::BottomToTop)
            .build();

        // Should contain ESC T 1 (set direction)
        assert!(page.windows(3).any(|w| w == [ESC, b'T', 1]));
    }

    #[test]
    fn page_with_vertical_position() {
        let page = PageBuilder::new()
            .vertical_position(256)
            .build();

        // Should contain GS $ 0 1 (position 256 = 0x0100)
        assert!(page.windows(4).any(|w| w == [GS, b'$', 0, 1]));
    }

    #[test]
    fn page_with_text() {
        let page = PageBuilder::new()
            .text("Hello")
            .build();

        assert!(page.windows(5).any(|w| w == b"Hello"));
    }

    #[test]
    fn page_with_raw_command() {
        use crate::command::paper::CutPaper;

        let page = PageBuilder::new()
            .command(&CutPaper::full())
            .build();

        // Should contain GS V 0 (full cut)
        assert!(page.windows(3).any(|w| w == [GS, b'V', 0]));
    }

    #[test]
    fn build_and_exit_adds_exit_command() {
        let page = PageBuilder::new().build_and_exit();

        // Should end with ESC S (exit page mode) after FF
        let len = page.len();
        assert_eq!(&page[len-2..], &[ESC, b'S']);
    }

    #[test]
    fn is_empty_and_len() {
        let empty = PageBuilder::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let with_text = PageBuilder::new().text("test");
        assert!(!with_text.is_empty());
        assert_eq!(with_text.len(), 1);
    }

    #[test]
    fn complex_page() {
        let page = PageBuilder::new()
            .area(PrintArea::default_80mm())
            .direction(PrintDirection::LeftToRight)
            .text_line("Header")
            .vertical_position(100)
            .text("Body")
            .build();

        // Verify structure
        let bytes = page;

        // Starts with enter page mode
        assert_eq!(&bytes[0..2], &[ESC, b'L']);

        // Contains set area
        assert!(bytes.windows(2).any(|w| w == [ESC, b'W']));

        // Contains set direction
        assert!(bytes.windows(2).any(|w| w == [ESC, b'T']));

        // Contains vertical position
        assert!(bytes.windows(2).any(|w| w == [GS, b'$']));

        // Contains text
        assert!(bytes.windows(6).any(|w| w == b"Header"));
        assert!(bytes.windows(4).any(|w| w == b"Body"));

        // Ends with form feed
        assert_eq!(bytes.last(), Some(&0x0C));
    }
}
```

---

## Commit

```bash
git add src/page/mod.rs src/lib.rs
git commit -m "feat: add PageBuilder for page mode command queuing"
```
