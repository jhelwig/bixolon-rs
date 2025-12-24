# bixolon

A Rust library for controlling Bixolon SRP-350plus (and compatible) thermal receipt printers using the ESC/POS command set.

[![CI](https://github.com/jhelwig/bixolon-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jhelwig/bixolon-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Transport-agnostic**: Works with any `Read`/`Write` implementation (files, USB, serial, TCP)
- **Styled text API**: Ergonomic builder pattern for bold, underline, italic, and other text styles
- **Page mode**: Build complex page layouts with precise positioning
- **Barcodes**: Support for Code128, Code39, EAN13, UPC-A, and other 1D barcode formats
- **2D symbols**: QR codes and PDF417 with configurable size and error correction
- **Status queries**: Read printer status (paper, drawer, errors) when using bidirectional transport
- **Async support**: Optional tokio-based async interface
- **USB support**: Optional direct USB communication via rusb

## Quick Start

```rust
use bixolon::prelude::*;
use bixolon::printer::Printer;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open printer device
    let file = File::create("/dev/usb/lp0")?;
    let mut printer = Printer::new(file);

    // Initialize printer
    printer.initialize()?;

    // Print styled text
    printer.println("Welcome!".bold())?;
    printer.println("Your receipt:".underlined())?;
    printer.println("Item 1       $10.00")?;
    printer.println("Item 2       $15.00")?;
    printer.println("Total".bold().append("        $25.00".bold()))?;

    // Cut paper
    printer.send(&CutPaper::feed_and_partial(3))?;
    printer.flush()?;

    Ok(())
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bixolon = "0.1"
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `async` | Enable async printer interface using tokio |
| `rusb` | Enable USB transport using rusb |
| `hardware-tests` | Enable tests that require a physical printer |

Example with USB support:

```toml
[dependencies]
bixolon = { version = "0.1", features = ["rusb"] }
```

## Usage Examples

### USB Connection

```rust
use bixolon::transport::usb::UsbPrinter;
use bixolon::printer::Printer;
use bixolon::prelude::*;

// Find and open the printer
let usb = UsbPrinter::find_bixolon()?.open()?;
let (reader, writer) = usb.split()?;
let mut printer = Printer::with_reader(writer, reader);

printer.initialize()?;
printer.println("Hello from USB!")?;
printer.flush()?;
```

### Styled Text

```rust
use bixolon::prelude::*;

// Simple styling
let text = "Important".bold();

// Combined styles
let text = "Bold and underlined".bold().underlined();

// Nested styling with different segments
let text = "Normal ".append("BOLD".bold()).append(" normal again");
```

### Barcodes

```rust
use bixolon::command::barcode::*;

// Configure barcode appearance
printer.send(&SetBarcodeHeight(100))?;
printer.send(&SetBarcodeWidth(BarcodeWidth::Width3))?;
printer.send(&SetHriPosition(HriPosition::Below))?;

// Print a Code128 barcode
let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A12345")?;
printer.send(&barcode)?;
```

### QR Codes

```rust
use bixolon::command::symbol::*;

let qr = PrintQrCode::new("https://example.com")?
    .with_module_size(QrModuleSize::Size4)?
    .with_error_correction(QrErrorCorrection::M);
printer.send(&qr)?;
```

### PDF417

```rust
use bixolon::command::symbol::*;

let pdf417 = PrintPdf417::new("Data to encode")?
    .with_module_size(Pdf417ModuleSize::Size3)?
    .with_columns(Pdf417Columns::Auto)?
    .with_error_correction(Pdf417ErrorCorrection::Level2);
printer.send(&pdf417)?;
```

### Page Mode

```rust
use bixolon::page::PageBuilder;
use bixolon::command::page_mode::PrintArea;

let page = PageBuilder::new()
    .area(PrintArea::default_80mm())
    .text_line("Header".bold())
    .vertical_position(200)
    .text_line("Body text")
    .build();

printer.send_raw(&page)?;
```

### Printer Status

```rust
use bixolon::command::status::*;

// Requires bidirectional connection (e.g., USB with reader)
let status = printer.query(&TransmitStatus(StatusType::Printer))?;
if let StatusResponse::Printer(ps) = status {
    if !ps.paper_present {
        println!("Out of paper!");
    }
}
```

## Architecture

The library is organized into layers:

| Layer | Module | Description |
|-------|--------|-------------|
| Command | `command` | Raw ESC/POS command structs |
| Style | `style` | Ergonomic styled text API |
| Page | `page` | Page mode command builder |
| Printer | `printer` | High-level printer interface |
| Transport | `transport` | USB/serial helpers (feature-gated) |

## Supported Hardware

- Bixolon SRP-350plus
- Other ESC/POS compatible thermal printers (may require testing)

## Requirements

- Rust 1.92.0 or later (edition 2024)
- libusb (for USB support on Linux): `apt install libusb-1.0-0-dev`

## License

MIT License. See [LICENSE](LICENSE) for details.
