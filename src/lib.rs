//! Bixolon thermal printer library for Rust.
//!
//! This crate provides a transport-agnostic interface for controlling
//! Bixolon SRP-350plus (and compatible) thermal receipt printers using
//! the ESC/POS command set.
//!
//! # Quick Start
//!
//! ```ignore
//! use bixolon::prelude::*;
//! use bixolon::printer::Printer;
//! use std::fs::File;
//!
//! // Open printer device
//! let file = File::create("/dev/usb/lp0")?;
//! let mut printer = Printer::new(file);
//!
//! // Initialize printer
//! printer.initialize()?;
//!
//! // Print styled text
//! printer.println("Welcome!".bold())?;
//! printer.println("Your receipt:".underlined())?;
//! printer.println("Item 1       $10.00")?;
//! printer.println("Item 2       $15.00")?;
//! printer.println("Total".bold().append("        $25.00".bold()))?;
//!
//! // Cut paper
//! printer.send(&CutPaper::feed_and_partial(3))?;
//! printer.flush()?;
//! ```
//!
//! # Architecture
//!
//! The library is organized into layers:
//!
//! - **Command layer** (`command` module): Raw ESC/POS command structs
//! - **Style layer** (`style` module): Ergonomic styled text API
//! - **Page layer** (`page` module): Page mode command builder
//! - **Printer layer** (`printer` module): High-level printer interface
//! - **Transport layer** (`transport` module): USB/serial helpers (feature-gated)
//!
//! # Features
//!
//! - `async` - Enable async printer interface using tokio
//! - `rusb` - Enable USB transport using rusb
//! - `hardware-tests` - Enable tests requiring physical printer
//!
//! # Styled Text
//!
//! The [`style`] module provides an ergonomic API for formatted text:
//!
//! ```ignore
//! use bixolon::prelude::*;
//!
//! // Simple styling
//! let text = "Important".bold();
//!
//! // Nested styling
//! let text = "Bold with ".bold().append("underline".underlined());
//!
//! // The style system handles nested styles correctly - when you
//! // exit an inner style, outer styles are preserved.
//! ```
//!
//! # Page Mode
//!
//! Use [`PageBuilder`] for page mode printing:
//!
//! ```ignore
//! use bixolon::prelude::*;
//! use bixolon::page::PageBuilder;
//! use bixolon::command::page_mode::PrintArea;
//!
//! let page = PageBuilder::new()
//!     .area(PrintArea::default_80mm())
//!     .text_line("Header".bold())
//!     .vertical_position(200)
//!     .text_line("Body text")
//!     .build();
//!
//! printer.send_raw(&page)?;
//! ```
//!
//! # Barcodes and QR Codes
//!
//! ```ignore
//! use bixolon::command::barcode::*;
//! use bixolon::command::symbol::*;
//!
//! // 1D Barcode
//! printer.send(&SetBarcodeHeight(100))?;
//! printer.send(&SetHriPosition(HriPosition::Below))?;
//! let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A12345")?;
//! printer.send(&barcode)?;
//!
//! // QR Code
//! let qr = PrintQrCode::new("https://example.com")?
//!     .with_module_size(4)?
//!     .with_error_correction(QrErrorCorrection::M);
//! printer.send(&qr)?;
//! ```
//!
//! # Status Queries
//!
//! When the printer is opened with a reader, status queries are available:
//!
//! ```ignore
//! use bixolon::command::status::*;
//!
//! let (reader, writer) = /* get USB handles */;
//! let mut printer = Printer::with_reader(writer, reader);
//!
//! let status = printer.query(&TransmitStatus(StatusType::Printer))?;
//! if let StatusResponse::Printer(ps) = status {
//!     if !ps.paper_present {
//!         println!("Out of paper!");
//!     }
//! }
//! ```

#![warn(missing_docs)]

pub mod command;
pub mod error;
pub mod page;
pub mod printer;
pub mod style;

#[cfg(feature = "rusb")]
pub mod transport;

/// Prelude module for convenient imports.
///
/// ```ignore
/// use bixolon::prelude::*;
/// ```
pub mod prelude {
    pub use crate::command::Command;
    pub use crate::command::paper::CutPaper;
    pub use crate::style::StyleSet;
    pub use crate::style::text::Styleable;

    #[cfg(feature = "async")]
    pub use crate::printer::AsyncPrinter;
    pub use crate::printer::Printer;
}

// Re-export commonly used types at crate root
pub use command::Command;
pub use error::{BarcodeError, PrinterError, QrCodeError};
pub use page::PageBuilder;
pub use printer::Printer;
pub use style::StyleSet;
pub use style::text::StyledNode;

#[cfg(feature = "async")]
pub use printer::AsyncPrinter;

#[cfg(feature = "rusb")]
pub use transport::usb::{UsbPrinter, UsbPrinterDevice};
