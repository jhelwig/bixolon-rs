# Task 17: Sync Printer

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/printer/mod.rs`
- Create: `src/printer/sync.rs`
- Modify: `src/lib.rs` (add module)

**Important:** `Printer<W, R>` where W: Write and R is optional reader. Query methods only available when `R: Read`.

---

## Core Design

1. **Transport agnostic**: Works with any `std::io::Write`
2. **Optional reader**: `R = ()` by default, enables status queries when `R: Read`
3. **Buffered writes**: Uses `BufWriter` for efficiency
4. **Explicit flush**: User controls when data is sent

---

## Implementation

```rust
// src/printer/mod.rs
//! Printer interface.
//!
//! Provides [`Printer`] for synchronous printing.

mod sync;

pub use sync::Printer;

#[cfg(feature = "async")]
mod r#async;

#[cfg(feature = "async")]
pub use r#async::AsyncPrinter;
```

```rust
// src/printer/sync.rs
//! Synchronous printer interface.

use std::io::{BufWriter, Read, Write};

use crate::command::{Command, QueryCommand};
use crate::command::printer_control::Initialize;
use crate::error::PrinterError;
use crate::page::PageBuilder;
use crate::style::text::StyledNode;

/// Synchronous printer interface.
///
/// Generic over the writer type `W` and optional reader type `R`.
/// When `R` implements `Read`, query methods become available.
///
/// # Example
///
/// ```ignore
/// use std::fs::File;
/// use bixolon::printer::Printer;
///
/// // Write-only printer (no status queries)
/// let file = File::create("/dev/usb/lp0")?;
/// let mut printer = Printer::new(file);
///
/// // With reader for status queries
/// let (reader, writer) = /* get USB handles */;
/// let mut printer = Printer::with_reader(writer, reader);
/// ```
pub struct Printer<W: Write, R = ()> {
    writer: BufWriter<W>,
    reader: R,
}

impl<W: Write> Printer<W, ()> {
    /// Create a new printer with write-only access.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            reader: (),
        }
    }
}

impl<W: Write, R> Printer<W, R> {
    /// Create a new printer with read and write access.
    pub fn with_reader(writer: W, reader: R) -> Self {
        Self {
            writer: BufWriter::new(writer),
            reader,
        }
    }

    /// Send a command to the printer.
    ///
    /// Does not flush - call `flush()` to ensure data is sent.
    pub fn send(&mut self, cmd: &impl Command) -> Result<(), PrinterError> {
        self.writer.write_all(&cmd.encode())?;
        Ok(())
    }

    /// Send raw bytes to the printer.
    ///
    /// Does not flush - call `flush()` to ensure data is sent.
    pub fn send_raw(&mut self, bytes: &[u8]) -> Result<(), PrinterError> {
        self.writer.write_all(bytes)?;
        Ok(())
    }

    /// Print styled text.
    ///
    /// Does not add a line feed. Use `println` for that.
    pub fn print(&mut self, text: impl Into<StyledNode>) -> Result<(), PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render())?;
        Ok(())
    }

    /// Print styled text followed by a line feed.
    pub fn println(&mut self, text: impl Into<StyledNode>) -> Result<(), PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render_line())?;
        Ok(())
    }

    /// Print a page mode document.
    pub fn print_page(&mut self, page: &PageBuilder) -> Result<(), PrinterError> {
        self.writer.write_all(&page.build())?;
        Ok(())
    }

    /// Print a page mode document and return to standard mode.
    pub fn print_page_and_exit(&mut self, page: &PageBuilder) -> Result<(), PrinterError> {
        self.writer.write_all(&page.build_and_exit())?;
        Ok(())
    }

    /// Initialize the printer (reset to defaults).
    pub fn initialize(&mut self) -> Result<(), PrinterError> {
        self.send(&Initialize)
    }

    /// Flush the write buffer to the printer.
    ///
    /// Call this to ensure all pending data is sent.
    pub fn flush(&mut self) -> Result<(), PrinterError> {
        self.writer.flush()?;
        Ok(())
    }

    /// Get a reference to the underlying writer.
    pub fn writer(&self) -> &W {
        self.writer.get_ref()
    }

    /// Get a mutable reference to the underlying writer.
    pub fn writer_mut(&mut self) -> &mut W {
        self.writer.get_mut()
    }

    /// Get a reference to the reader (if any).
    pub fn reader(&self) -> &R {
        &self.reader
    }

    /// Get a mutable reference to the reader (if any).
    pub fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consume the printer and return the inner writer and reader.
    pub fn into_inner(self) -> (W, R) {
        (self.writer.into_inner().expect("flush failed"), self.reader)
    }
}

impl<W: Write, R: Read> Printer<W, R> {
    /// Execute a query command and parse the response.
    ///
    /// Flushes the write buffer before reading the response.
    pub fn query<Q: QueryCommand>(&mut self, cmd: &Q) -> Result<Q::Response, PrinterError> {
        // Send the query command
        self.writer.write_all(&cmd.encode())?;
        self.writer.flush()?;

        // Read response
        let mut buf = [0u8; 64];
        let n = self.reader.read(&mut buf)?;

        if n == 0 {
            return Err(PrinterError::NoResponse);
        }

        cmd.parse_response(&buf[..n])
            .map_err(PrinterError::StatusParse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn new_creates_write_only() {
        let buf = Vec::new();
        let _printer: Printer<Vec<u8>, ()> = Printer::new(buf);
    }

    #[test]
    fn with_reader_creates_read_write() {
        let writer = Vec::new();
        let reader = Cursor::new(vec![0u8; 10]);
        let _printer: Printer<Vec<u8>, Cursor<Vec<u8>>> = Printer::with_reader(writer, reader);
    }

    #[test]
    fn send_writes_command() {
        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        printer.send(&Initialize).unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();
        assert_eq!(inner, vec![0x1B, b'@']);
    }

    #[test]
    fn send_raw_writes_bytes() {
        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        printer.send_raw(&[0x1B, b'@']).unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();
        assert_eq!(inner, vec![0x1B, b'@']);
    }

    #[test]
    fn print_writes_styled_text() {
        use crate::style::text::Styleable;

        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        printer.print("Hello".bold()).unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();

        // Should contain bold on, text, bold off
        assert!(inner.windows(3).any(|w| w == [0x1B, b'E', 1]));
        assert!(inner.windows(5).any(|w| w == b"Hello"));
        assert!(inner.windows(3).any(|w| w == [0x1B, b'E', 0]));
    }

    #[test]
    fn println_appends_lf() {
        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        printer.println("Hello").unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();
        assert!(inner.ends_with(&[0x0A]));
    }

    #[test]
    fn print_page_sends_page_mode() {
        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        let page = PageBuilder::new().text("Test");
        printer.print_page(&page).unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();

        // Should start with ESC L
        assert_eq!(&inner[0..2], &[0x1B, b'L']);
        // Should end with FF
        assert!(inner.ends_with(&[0x0C]));
    }

    #[test]
    fn initialize_sends_esc_at() {
        let buf = Vec::new();
        let mut printer = Printer::new(buf);

        printer.initialize().unwrap();
        printer.flush().unwrap();

        let (inner, _) = printer.into_inner();
        assert_eq!(inner, vec![0x1B, b'@']);
    }

    #[test]
    fn query_requires_reader() {
        use crate::command::status::{TransmitStatus, StatusType};

        let writer = Vec::new();
        let reader = Cursor::new(vec![0x00u8]); // Simulated response
        let mut printer = Printer::with_reader(writer, reader);

        let result = printer.query(&TransmitStatus(StatusType::Printer));
        assert!(result.is_ok());
    }
}
```

---

## Add to error.rs

Add these variants to the error module:

```rust
/// Printer operation errors.
#[derive(Debug, thiserror::Error)]
pub enum PrinterError {
    /// IO error during communication.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// No response received from printer.
    #[error("no response received from printer")]
    NoResponse,

    /// Failed to parse status response.
    #[error("failed to parse status: {0}")]
    StatusParse(#[from] StatusParseError),
}
```

---

## Commit

```bash
git add src/printer/mod.rs src/printer/sync.rs src/error.rs src/lib.rs
git commit -m "feat: add synchronous Printer interface"
```
