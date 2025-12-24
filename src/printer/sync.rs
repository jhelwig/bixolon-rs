//! Synchronous printer interface.

use std::io::{BufWriter, Read, Write};

use crate::command::printer_control::Initialize;
use crate::command::{Command, QueryCommand};
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
    pub fn send(&mut self, cmd: &impl Command) -> Result<&mut Self, PrinterError> {
        self.writer.write_all(&cmd.encode())?;
        Ok(self)
    }

    /// Send raw bytes to the printer.
    ///
    /// Does not flush - call `flush()` to ensure data is sent.
    pub fn send_raw(&mut self, bytes: &[u8]) -> Result<&mut Self, PrinterError> {
        self.writer.write_all(bytes)?;
        Ok(self)
    }

    /// Print styled text.
    ///
    /// Does not add a line feed. Use `println` for that.
    pub fn print(&mut self, text: impl Into<StyledNode>) -> Result<&mut Self, PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render())?;
        Ok(self)
    }

    /// Print styled text followed by a line feed.
    pub fn println(&mut self, text: impl Into<StyledNode>) -> Result<&mut Self, PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render_line())?;
        Ok(self)
    }

    /// Print a page mode document.
    pub fn print_page(&mut self, page: &PageBuilder) -> Result<&mut Self, PrinterError> {
        self.writer.write_all(&page.build())?;
        Ok(self)
    }

    /// Print a page mode document and return to standard mode.
    pub fn print_page_and_exit(&mut self, page: &PageBuilder) -> Result<&mut Self, PrinterError> {
        self.writer.write_all(&page.build_and_exit())?;
        Ok(self)
    }

    /// Initialize the printer (reset to defaults).
    pub fn initialize(&mut self) -> Result<&mut Self, PrinterError> {
        self.send(&Initialize)
    }

    /// Flush the write buffer to the printer.
    ///
    /// Call this to ensure all pending data is sent.
    pub fn flush(&mut self) -> Result<&mut Self, PrinterError> {
        self.writer.flush()?;
        Ok(self)
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
    ///
    /// # Panics
    ///
    /// Panics if the buffer cannot be flushed. Call `flush()` before
    /// `into_inner()` if you need to handle flush errors gracefully.
    pub fn into_inner(self) -> (W, R) {
        // Use ok() to avoid the Debug bound on IntoInnerError
        match self.writer.into_inner() {
            Ok(writer) => (writer, self.reader),
            Err(err) => {
                // The buffer failed to flush. We still return the writer
                // to allow recovery, but we can't return the error without
                // requiring W: Debug. Instead we panic with a message.
                let _ = err; // Silence unused warning
                panic!("failed to flush printer buffer on into_inner")
            }
        }
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

        cmd.parse_response(&buf[..n]).map_err(PrinterError::StatusParse)
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
        use crate::command::status::{StatusType, TransmitStatus};

        let writer = Vec::new();
        let reader = Cursor::new(vec![0x00u8]); // Simulated response
        let mut printer = Printer::with_reader(writer, reader);

        let result = printer.query(&TransmitStatus(StatusType::Printer));
        assert!(result.is_ok());
    }
}
