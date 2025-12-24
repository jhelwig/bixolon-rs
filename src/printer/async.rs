//! Asynchronous printer interface.
//!
//! Requires the `async` feature.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

use crate::command::printer_control::Initialize;
use crate::command::{Command, QueryCommand};
use crate::error::PrinterError;
use crate::page::PageBuilder;
use crate::style::text::StyledNode;

/// Asynchronous printer interface.
///
/// Generic over the writer type `W` and optional reader type `R`.
/// When `R` implements `AsyncRead`, query methods become available.
///
/// # Example
///
/// ```ignore
/// use bixolon::printer::AsyncPrinter;
///
/// // Write-only printer
/// let writer = /* async writer */;
/// let mut printer = AsyncPrinter::new(writer);
///
/// // With reader for status queries
/// let (reader, writer) = /* get async handles */;
/// let mut printer = AsyncPrinter::with_reader(writer, reader);
/// ```
pub struct AsyncPrinter<W: AsyncWrite + Unpin, R = ()> {
    writer: BufWriter<W>,
    reader: R,
}

impl<W: AsyncWrite + Unpin> AsyncPrinter<W, ()> {
    /// Create a new printer with write-only access.
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            reader: (),
        }
    }
}

impl<W: AsyncWrite + Unpin, R> AsyncPrinter<W, R> {
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
    pub async fn send(&mut self, cmd: &impl Command) -> Result<&Self, PrinterError> {
        self.writer.write_all(&cmd.encode()).await?;
        Ok(self)
    }

    /// Send raw bytes to the printer.
    ///
    /// Does not flush - call `flush()` to ensure data is sent.
    pub async fn send_raw(&mut self, bytes: &[u8]) -> Result<&Self, PrinterError> {
        self.writer.write_all(bytes).await?;
        Ok(self)
    }

    /// Print styled text.
    ///
    /// Does not add a line feed. Use `println` for that.
    pub async fn print(&mut self, text: impl Into<StyledNode>) -> Result<&Self, PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render()).await?;
        Ok(self)
    }

    /// Print styled text followed by a line feed.
    pub async fn println(&mut self, text: impl Into<StyledNode>) -> Result<&Self, PrinterError> {
        let node = text.into();
        self.writer.write_all(&node.render_line()).await?;
        Ok(self)
    }

    /// Print a page mode document.
    pub async fn print_page(&mut self, page: &PageBuilder) -> Result<&Self, PrinterError> {
        self.writer.write_all(&page.build()).await?;
        Ok(self)
    }

    /// Print a page mode document and return to standard mode.
    pub async fn print_page_and_exit(&mut self, page: &PageBuilder) -> Result<&Self, PrinterError> {
        self.writer.write_all(&page.build_and_exit()).await?;
        Ok(self)
    }

    /// Initialize the printer (reset to defaults).
    pub async fn initialize(&mut self) -> Result<&Self, PrinterError> {
        self.send(&Initialize).await?;
        Ok(self)
    }

    /// Flush the write buffer to the printer.
    ///
    /// Call this to ensure all pending data is sent.
    pub async fn flush(&mut self) -> Result<&Self, PrinterError> {
        self.writer.flush().await?;
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
    pub fn into_inner(self) -> (W, R) {
        (self.writer.into_inner(), self.reader)
    }
}

impl<W: AsyncWrite + Unpin, R: AsyncRead + Unpin> AsyncPrinter<W, R> {
    /// Execute a query command and parse the response.
    ///
    /// Flushes the write buffer before reading the response.
    pub async fn query<Q: QueryCommand>(&mut self, cmd: &Q) -> Result<Q::Response, PrinterError> {
        // Send the query command
        self.writer.write_all(&cmd.encode()).await?;
        self.writer.flush().await?;

        // Read response
        let mut buf = [0u8; 64];
        let n = self.reader.read(&mut buf).await?;

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

    // Helper to create async cursor
    fn async_cursor(data: Vec<u8>) -> Cursor<Vec<u8>> {
        Cursor::new(data)
    }

    #[tokio::test]
    async fn new_creates_write_only() {
        let buf = async_cursor(Vec::new());
        let _printer: AsyncPrinter<Cursor<Vec<u8>>, ()> = AsyncPrinter::new(buf);
    }

    #[tokio::test]
    async fn send_writes_command() {
        let buf = async_cursor(Vec::new());
        let mut printer = AsyncPrinter::new(buf);

        printer.send(&Initialize).await.unwrap();
        printer.flush().await.unwrap();

        let (inner, _) = printer.into_inner();
        assert_eq!(inner.into_inner(), vec![0x1B, b'@']);
    }

    #[tokio::test]
    async fn println_appends_lf() {
        let buf = async_cursor(Vec::new());
        let mut printer = AsyncPrinter::new(buf);

        printer.println("Hello").await.unwrap();
        printer.flush().await.unwrap();

        let (inner, _) = printer.into_inner();
        let bytes = inner.into_inner();
        assert!(bytes.ends_with(&[0x0A]));
    }

    #[tokio::test]
    async fn print_page_sends_page_mode() {
        let buf = async_cursor(Vec::new());
        let mut printer = AsyncPrinter::new(buf);

        let page = PageBuilder::new().text("Test");
        printer.print_page(&page).await.unwrap();
        printer.flush().await.unwrap();

        let (inner, _) = printer.into_inner();
        let bytes = inner.into_inner();

        // Should start with ESC L
        assert_eq!(&bytes[0..2], &[0x1B, b'L']);
        // Should end with FF
        assert!(bytes.ends_with(&[0x0C]));
    }

    #[tokio::test]
    async fn query_requires_reader() {
        use crate::command::status::{StatusType, TransmitStatus};

        let writer = async_cursor(Vec::new());
        let reader = async_cursor(vec![0x00u8]); // Simulated response
        let mut printer = AsyncPrinter::with_reader(writer, reader);

        let result = printer.query(&TransmitStatus(StatusType::Printer)).await;
        assert!(result.is_ok());
    }
}
