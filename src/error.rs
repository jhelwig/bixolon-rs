//! Error types for the Bixolon printer library.

// The miette derive macro reads these fields at runtime for diagnostics,
// but clippy's static analysis doesn't see this usage.
#![allow(unused_assignments)]

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Result type alias using PrinterError.
pub type Result<T> = std::result::Result<T, PrinterError>;

/// Top-level error type for all printer operations.
#[derive(Debug, Error)]
pub enum PrinterError {
    /// IO error during communication with the printer.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Text encoding error - character not representable in code page.
    #[error("encoding error")]
    Encoding(#[from] EncodingError),

    /// Command validation error.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Barcode-specific error.
    #[error("barcode error")]
    Barcode(#[from] BarcodeError),

    /// Printer reported an error status.
    #[error("printer status error: {0}")]
    Status(#[from] StatusError),

    /// Parse error when reading printer response.
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    /// No reader configured for bidirectional communication.
    #[error("no reader configured for bidirectional communication")]
    NoReader,

    /// No response received from printer.
    #[error("no response received from printer")]
    NoResponse,

    /// Failed to parse status response.
    #[error("failed to parse status: {0}")]
    StatusParse(#[source] StatusParseError),
}

/// Encoding error with source span for miette diagnostics.
#[derive(Debug, Error, Diagnostic)]
#[error("character not representable in {code_page}")]
#[diagnostic(code(bixolon::encoding))]
pub struct EncodingError {
    /// The source text being encoded.
    #[source_code]
    pub src: String,

    /// Span pointing to the problematic character.
    #[label("this character cannot be encoded")]
    pub span: SourceSpan,

    /// The code page being used.
    pub code_page: String,

    /// Optional help message.
    #[help]
    pub help: Option<String>,
}

/// Validation error for command parameters.
#[derive(Debug, Error, Diagnostic)]
pub enum ValidationError {
    /// Line spacing value out of range.
    #[error("line spacing value {0} out of range")]
    #[diagnostic(code(bixolon::validation::line_spacing))]
    InvalidLineSpacing(u8),

    /// Tab position out of range.
    #[error("tab position {0} out of range")]
    #[diagnostic(code(bixolon::validation::tab))]
    InvalidTabPosition(u8),

    /// Invalid code page value.
    #[error("invalid code page value: {0}")]
    #[diagnostic(code(bixolon::validation::codepage))]
    InvalidCodePage(u8),

    /// Generic parameter out of range.
    #[error("{name} value {value} out of range ({min}-{max})")]
    #[diagnostic(code(bixolon::validation::range))]
    OutOfRange {
        /// The name of the parameter.
        name: &'static str,
        /// The value that was out of range.
        value: u16,
        /// The minimum allowed value.
        min: u16,
        /// The maximum allowed value.
        max: u16,
    },
}

/// Barcode-specific errors with source spans.
#[derive(Debug, Error, Diagnostic)]
pub enum BarcodeError {
    /// Invalid barcode data length.
    #[error("invalid barcode length for {system}: got {actual}, expected {min}-{max}")]
    #[diagnostic(code(bixolon::barcode::length))]
    InvalidLength {
        /// The barcode system name.
        system: &'static str,
        /// The actual data length.
        actual: usize,
        /// The minimum allowed length.
        min: usize,
        /// The maximum allowed length.
        max: usize,
    },

    /// ITF barcode requires even number of digits.
    #[error("ITF barcode requires even number of digits, got {0}")]
    #[diagnostic(code(bixolon::barcode::itf_length))]
    ItfRequiresEvenLength(usize),

    /// Invalid character in barcode data.
    #[error("invalid character in {system} barcode")]
    #[diagnostic(code(bixolon::barcode::character))]
    InvalidCharacter {
        /// The barcode data.
        #[source_code]
        data: String,

        /// Span pointing to the invalid character.
        #[label("invalid character")]
        span: SourceSpan,

        /// The barcode system.
        system: &'static str,
    },
}

/// Printer status errors.
#[derive(Debug, Error)]
pub enum StatusError {
    /// Printer is offline.
    #[error("printer is offline")]
    Offline,

    /// Paper end detected.
    #[error("paper end detected")]
    PaperEnd,

    /// Paper near end.
    #[error("paper near end")]
    PaperNearEnd,

    /// Cover is open.
    #[error("cover is open")]
    CoverOpen,

    /// Cutter error.
    #[error("cutter error")]
    CutterError,

    /// Mechanical error.
    #[error("mechanical error")]
    MechanicalError,

    /// Unrecoverable error.
    #[error("unrecoverable error")]
    UnrecoverableError,
}

/// Parse error when reading printer response.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Unexpected response length.
    #[error("unexpected response length: expected {expected}, got {actual}")]
    UnexpectedLength {
        /// The expected response length.
        expected: usize,
        /// The actual response length.
        actual: usize,
    },

    /// Invalid response format.
    #[error("invalid response format: {0}")]
    InvalidFormat(String),

    /// Unknown status byte.
    #[error("unknown status byte: {0:#04x}")]
    UnknownStatus(u8),
}

/// Status parse error.
#[derive(Debug, Error)]
pub enum StatusParseError {
    /// Empty response from printer.
    #[error("empty response from printer")]
    EmptyResponse,

    /// Invalid status byte.
    #[error("invalid status byte: {0:#04x}")]
    InvalidStatus(u8),
}

/// QR Code errors.
#[derive(Debug, Error)]
pub enum QrCodeError {
    /// Data is empty.
    #[error("QR code data cannot be empty")]
    EmptyData,

    /// Data exceeds maximum length.
    #[error("QR code data too long: {0} bytes (max 7089)")]
    DataTooLong(usize),
}

/// PDF417 errors.
#[derive(Debug, Error)]
pub enum Pdf417Error {
    /// Invalid column count.
    #[error("invalid PDF417 column count {0} (must be 1-30)")]
    InvalidColumnCount(u8),

    /// Invalid row count.
    #[error("invalid PDF417 row count {0} (must be 3-90)")]
    InvalidRowCount(u8),
}

/// USB transport errors.
#[cfg(feature = "rusb")]
#[derive(Debug, Error)]
pub enum UsbError {
    /// rusb error.
    #[error("USB error: {0}")]
    Rusb(#[from] rusb::Error),

    /// Device not found.
    #[error("USB printer device not found")]
    DeviceNotFound,

    /// No write endpoint found.
    #[error("no bulk OUT endpoint found")]
    NoWriteEndpoint,

    /// No read endpoint found.
    #[error("no bulk IN endpoint found")]
    NoReadEndpoint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printer_error_displays_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "device not found");
        let err: PrinterError = io_err.into();
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn encoding_error_has_span() {
        let err = EncodingError {
            src: "Hello 世界".to_string(),
            span: (6, 3).into(),
            code_page: "CP437".to_string(),
            help: Some("Use a code page that supports CJK characters".to_string()),
        };
        assert_eq!(err.src, "Hello 世界");
    }

    #[test]
    fn barcode_error_invalid_character() {
        let err = BarcodeError::InvalidCharacter {
            data: "123ABC456".to_string(),
            span: (3, 3).into(),
            system: "UPC-A",
        };
        assert!(err.to_string().contains("UPC-A"));
    }
}
