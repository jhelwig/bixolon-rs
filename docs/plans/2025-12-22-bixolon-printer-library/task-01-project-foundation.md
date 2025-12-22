# Task 1: Project Foundation

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs` (minimal, just module declarations)
- Create: `src/error.rs`
- Create: `src/command/mod.rs`

---

## Step 1: Create Cargo.toml

Create `Cargo.toml` in project root:

```toml
[package]
name = "bixolon"
version = "0.1.0"
edition = "2024"
rust-version = "1.92.0"
license = "MIT"
description = "Rust library for controlling Bixolon SRP-350plus thermal receipt printers"
repository = "https://github.com/jhelwig/bixolon-rs"
keywords = ["printer", "thermal", "receipt", "pos", "escpos"]
categories = ["hardware-support"]

[features]
default = []
async = ["dep:tokio"]
rusb = ["dep:rusb"]
hardware-tests = []

[dependencies]
thiserror = "2"
miette = { version = "7", features = ["fancy"] }

# Optional async runtime
tokio = { version = "1", features = ["io-util"], optional = true }

# Optional USB support
rusb = { version = "0.9", optional = true }

[dev-dependencies]
```

---

## Step 2: Create minimal src/lib.rs

Create `src/lib.rs`:

```rust
//! Bixolon SRP-350plus thermal receipt printer library.
//!
//! This library provides complete ESC/POS command set support with ergonomic APIs.

pub mod command;
pub mod error;

pub use error::{PrinterError, Result};
```

---

## Step 3: Write failing test for error types

Create `src/error.rs`:

```rust
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
        // Just verify it compiles and has the right fields
        assert_eq!(err.src, "Hello 世界");
    }
}
```

---

## Step 4: Run test to verify it fails

```bash
cargo test -p bixolon
```

Expected: Compilation error - `PrinterError`, `EncodingError` not defined

---

## Step 5: Implement error types

Update `src/error.rs`:

```rust
//! Error types for the Bixolon printer library.

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
    /// Character scale factor out of range.
    #[error("invalid character scale factor: width={width}, height={height} (must be 1-8)")]
    #[diagnostic(code(bixolon::validation::scale))]
    InvalidScaleFactor {
        width: u8,
        height: u8,
    },

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
        name: &'static str,
        value: u16,
        min: u16,
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
        system: &'static str,
        actual: usize,
        min: usize,
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
    UnexpectedLength { expected: usize, actual: usize },

    /// Invalid response format.
    #[error("invalid response format: {0}")]
    InvalidFormat(String),

    /// Unknown status byte.
    #[error("unknown status byte: {0:#04x}")]
    UnknownStatus(u8),
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

    #[test]
    fn validation_error_scale_factor() {
        let err = ValidationError::InvalidScaleFactor { width: 9, height: 1 };
        assert!(err.to_string().contains("width=9"));
    }
}
```

---

## Step 6: Run tests to verify they pass

```bash
cargo test -p bixolon
```

Expected: All tests pass

---

## Step 7: Write failing test for Command trait

Create `src/command/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_constant_is_0x1b() {
        assert_eq!(ESC, 0x1B);
    }

    #[test]
    fn gs_constant_is_0x1d() {
        assert_eq!(GS, 0x1D);
    }
}
```

---

## Step 8: Run test to verify it fails

```bash
cargo test -p bixolon
```

Expected: Compilation error - `ESC`, `GS` not defined

---

## Step 9: Implement Command trait and constants

Update `src/command/mod.rs`:

```rust
//! ESC/POS command definitions.
//!
//! All commands implement the [`Command`] trait for encoding to bytes.

/// Escape character - starts most ESC/POS commands.
pub const ESC: u8 = 0x1B;

/// Group Separator - starts GS commands.
pub const GS: u8 = 0x1D;

/// File Separator - starts FS commands.
pub const FS: u8 = 0x1C;

/// Data Link Escape - starts real-time commands.
pub const DLE: u8 = 0x10;

/// End of Transmission - used in real-time status commands.
pub const EOT: u8 = 0x04;

/// Line Feed.
pub const LF: u8 = 0x0A;

/// Form Feed.
pub const FF: u8 = 0x0C;

/// Carriage Return.
pub const CR: u8 = 0x0D;

/// Horizontal Tab.
pub const HT: u8 = 0x09;

/// Cancel.
pub const CAN: u8 = 0x18;

/// A command that can be sent to the printer.
///
/// Commands serialize to byte sequences in ESC/POS format.
pub trait Command {
    /// Encode this command to bytes.
    fn encode(&self) -> Vec<u8>;
}

/// A command that expects a response from the printer.
pub trait QueryCommand: Command {
    /// The type of response expected.
    type Response;

    /// Parse the response bytes into the expected type.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`](crate::error::ParseError) if the response is malformed.
    fn parse_response(&self, bytes: &[u8]) -> Result<Self::Response, crate::error::ParseError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_constant_is_0x1b() {
        assert_eq!(ESC, 0x1B);
    }

    #[test]
    fn gs_constant_is_0x1d() {
        assert_eq!(GS, 0x1D);
    }

    #[test]
    fn fs_constant_is_0x1c() {
        assert_eq!(FS, 0x1C);
    }

    #[test]
    fn dle_constant_is_0x10() {
        assert_eq!(DLE, 0x10);
    }

    #[test]
    fn lf_constant_is_0x0a() {
        assert_eq!(LF, 0x0A);
    }

    #[test]
    fn ff_constant_is_0x0c() {
        assert_eq!(FF, 0x0C);
    }
}
```

---

## Step 10: Run tests to verify they pass

```bash
cargo test -p bixolon
```

Expected: All tests pass

---

## Step 11: Commit

```bash
git add Cargo.toml src/
git commit -m "feat: add project foundation with error types and Command trait"
```
