# Task 22: Hardware Tests

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `tests/hardware.rs`

**CRITICAL:** Hardware tests require `BIXOLON_TEST_DEVICE` environment variable. Tests **FAIL** (not skip) if not set. This ensures CI catches missing configuration rather than silently passing.

---

## Goals

1. Test with real hardware
2. Print test identification for manual verification
3. Test all major features: text, styles, barcodes, QR codes, page mode
4. Require explicit opt-in via feature flag

---

## Implementation

```rust
//! Hardware tests for bixolon library.
//!
//! These tests require a physical printer connected.
//!
//! # Requirements
//!
//! - Set `BIXOLON_TEST_DEVICE` environment variable to the device path
//!   (e.g., `/dev/usb/lp0` or `/dev/ttyUSB0`)
//! - Run with `--features hardware-tests`
//!
//! # Running
//!
//! ```bash
//! BIXOLON_TEST_DEVICE=/dev/usb/lp0 cargo test --features hardware-tests -- --test-threads=1
//! ```
//!
//! **Important:** Use `--test-threads=1` to prevent concurrent printer access.

#![cfg(feature = "hardware-tests")]

use std::fs::OpenOptions;

use bixolon::command::*;
use bixolon::command::basic::*;
use bixolon::command::character::*;
use bixolon::command::paper::*;
use bixolon::command::barcode::*;
use bixolon::command::symbol::*;
use bixolon::command::page_mode::*;
use bixolon::page::PageBuilder;
use bixolon::printer::Printer;
use bixolon::style::text::Styleable;

/// Get the test device path or panic.
///
/// Tests FAIL if BIXOLON_TEST_DEVICE is not set.
fn get_device_path() -> String {
    std::env::var("BIXOLON_TEST_DEVICE").expect(
        "BIXOLON_TEST_DEVICE environment variable must be set to run hardware tests. \
         Example: BIXOLON_TEST_DEVICE=/dev/usb/lp0"
    )
}

/// Open a printer for testing.
fn open_printer() -> Printer<std::fs::File, ()> {
    let path = get_device_path();
    let file = OpenOptions::new()
        .write(true)
        .open(&path)
        .expect(&format!("Failed to open printer device: {}", path));
    Printer::new(file)
}

/// Print a test header identifying the test.
fn print_test_header(printer: &mut Printer<std::fs::File, ()>, test_name: &str, description: &str) {
    printer.send(&bixolon::command::printer_control::Initialize).unwrap();
    printer.send(&SetJustification(Justification::Center)).unwrap();
    printer.println("=== HARDWARE TEST ===".bold()).unwrap();
    printer.println(test_name.underlined()).unwrap();
    printer.send(&SetJustification(Justification::Left)).unwrap();
    printer.println(description).unwrap();
    printer.send(&FeedLines(1)).unwrap();
}

/// Print a test footer with verification instructions.
fn print_test_footer(printer: &mut Printer<std::fs::File, ()>, expected: &str) {
    printer.send(&FeedLines(1)).unwrap();
    printer.println("Expected:").unwrap();
    printer.println(expected).unwrap();
    printer.send(&FeedLines(2)).unwrap();
    printer.send(&CutPaper::partial()).unwrap();
    printer.flush().unwrap();
}

// ============================================================================
// Basic Printing
// ============================================================================

#[test]
fn hardware_basic_text() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "basic_text",
        "Tests plain text printing",
    );

    printer.println("Line 1: Plain text").unwrap();
    printer.println("Line 2: More text").unwrap();
    printer.println("Line 3: Final line").unwrap();

    print_test_footer(
        &mut printer,
        "Three lines of plain text",
    );
}

#[test]
fn hardware_styled_text() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "styled_text",
        "Tests various text styles",
    );

    printer.println("Normal text").unwrap();
    printer.println("Bold text".bold()).unwrap();
    printer.println("Underlined text".underlined()).unwrap();
    printer.println("Double underline".double_underlined()).unwrap();
    printer.println("Reversed text".reversed()).unwrap();
    printer.println("Combined".bold().append(" styles".underlined())).unwrap();

    print_test_footer(
        &mut printer,
        "Normal, Bold, Underline, Double-underline, Reversed, Combined",
    );
}

#[test]
fn hardware_nested_styles() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "nested_styles",
        "Tests nested style handling",
    );

    // This tests the critical case: inner underline, outer bold
    // When underline ends, bold should continue
    let text = "outer bold [".bold()
        .append("inner underline".underlined())
        .append("] still bold".bold());
    printer.println(text).unwrap();

    print_test_footer(
        &mut printer,
        "Bold text with underlined section in middle, bold continues after",
    );
}

// ============================================================================
// Justification
// ============================================================================

#[test]
fn hardware_justification() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "justification",
        "Tests text alignment",
    );

    printer.send(&SetJustification(Justification::Left)).unwrap();
    printer.println("Left aligned").unwrap();

    printer.send(&SetJustification(Justification::Center)).unwrap();
    printer.println("Center aligned").unwrap();

    printer.send(&SetJustification(Justification::Right)).unwrap();
    printer.println("Right aligned").unwrap();

    printer.send(&SetJustification(Justification::Left)).unwrap();

    print_test_footer(
        &mut printer,
        "Left, Center, Right aligned text",
    );
}

// ============================================================================
// Barcodes
// ============================================================================

#[test]
fn hardware_barcode_code128() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "barcode_code128",
        "Tests CODE128 barcode",
    );

    printer.send(&SetBarcodeHeight(80)).unwrap();
    printer.send(&SetBarcodeWidth(BarcodeWidth::Normal)).unwrap();
    printer.send(&SetHriPosition(HriPosition::Below)).unwrap();
    printer.send(&SetHriFont(HriFont::A)).unwrap();

    let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A12345").unwrap();
    printer.send(&barcode).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    print_test_footer(
        &mut printer,
        "CODE128 barcode with '12345' below",
    );
}

#[test]
fn hardware_barcode_upc_a() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "barcode_upc_a",
        "Tests UPC-A barcode",
    );

    printer.send(&SetBarcodeHeight(100)).unwrap();
    printer.send(&SetHriPosition(HriPosition::Below)).unwrap();

    let barcode = PrintBarcode::new(BarcodeSystem::UpcA, b"01234567890").unwrap();
    printer.send(&barcode).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    print_test_footer(
        &mut printer,
        "UPC-A barcode for 012345678905 (with check digit)",
    );
}

// ============================================================================
// QR Codes
// ============================================================================

#[test]
fn hardware_qr_code() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "qr_code",
        "Tests QR code generation",
    );

    let qr = PrintQrCode::new(b"https://example.com".to_vec())
        .unwrap()
        .with_module_size(4)
        .unwrap()
        .with_error_correction(QrErrorCorrection::M);

    printer.send(&qr).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    print_test_footer(
        &mut printer,
        "QR code that scans to https://example.com",
    );
}

#[test]
fn hardware_qr_code_sizes() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "qr_code_sizes",
        "Tests QR code module sizes",
    );

    for size in [2, 4, 6, 8] {
        printer.println(format!("Size {}:", size)).unwrap();
        let qr = PrintQrCode::new(b"TEST".to_vec())
            .unwrap()
            .with_module_size(size)
            .unwrap();
        printer.send(&qr).unwrap();
        printer.send(&FeedLines(1)).unwrap();
    }

    print_test_footer(
        &mut printer,
        "Four QR codes of increasing size (2, 4, 6, 8)",
    );
}

// ============================================================================
// Page Mode
// ============================================================================

#[test]
fn hardware_page_mode_basic() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "page_mode_basic",
        "Tests basic page mode",
    );

    let page = PageBuilder::new()
        .area(PrintArea::default_80mm())
        .text_line("Page Mode Header".bold())
        .text_line("Line 1")
        .text_line("Line 2")
        .text_line("Line 3");

    printer.print_page_and_exit(&page).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    print_test_footer(
        &mut printer,
        "Bold header followed by three lines",
    );
}

#[test]
fn hardware_page_mode_positioning() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "page_mode_positioning",
        "Tests vertical positioning in page mode",
    );

    let page = PageBuilder::new()
        .area(PrintArea { x: 0, y: 0, width: 512, height: 600 })
        .vertical_position(0)
        .text_line("Position 0")
        .vertical_position(100)
        .text_line("Position 100")
        .vertical_position(200)
        .text_line("Position 200");

    printer.print_page_and_exit(&page).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    print_test_footer(
        &mut printer,
        "Three lines at different vertical positions",
    );
}

// ============================================================================
// Paper Handling
// ============================================================================

#[test]
fn hardware_paper_feed() {
    let mut printer = open_printer();

    print_test_header(
        &mut printer,
        "paper_feed",
        "Tests paper feed commands",
    );

    printer.println("Before feed").unwrap();
    printer.send(&FeedPaper(50)).unwrap(); // 50 dots
    printer.println("After 50 dots").unwrap();
    printer.send(&FeedLines(3)).unwrap(); // 3 lines
    printer.println("After 3 lines").unwrap();

    print_test_footer(
        &mut printer,
        "Text with gaps: 50 dots, then 3 lines",
    );
}

#[test]
fn hardware_cut_modes() {
    let mut printer = open_printer();

    // Note: This test cuts paper multiple times
    print_test_header(
        &mut printer,
        "cut_modes",
        "Tests different cut modes",
    );

    printer.println("Full cut next:").unwrap();
    printer.send(&FeedLines(2)).unwrap();
    printer.send(&CutPaper::full()).unwrap();

    // Give printer time to complete cut
    std::thread::sleep(std::time::Duration::from_millis(500));

    printer.println("Partial cut next:").unwrap();
    printer.send(&FeedLines(2)).unwrap();
    printer.send(&CutPaper::partial()).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));

    printer.println("Feed and partial:").unwrap();
    printer.send(&CutPaper::feed_and_partial(5)).unwrap();

    printer.flush().unwrap();
}

// ============================================================================
// Complete Receipt
// ============================================================================

#[test]
fn hardware_complete_receipt() {
    let mut printer = open_printer();

    printer.send(&bixolon::command::printer_control::Initialize).unwrap();

    // Header
    printer.send(&SetJustification(Justification::Center)).unwrap();
    printer.println("BIXOLON TEST RECEIPT".bold()).unwrap();
    printer.println("Hardware Test Suite").unwrap();
    printer.send(&SetJustification(Justification::Left)).unwrap();

    printer.println("================================").unwrap();

    // Items
    printer.println("Item 1              $10.00").unwrap();
    printer.println("Item 2              $15.00").unwrap();
    printer.println("Item 3               $5.00").unwrap();

    printer.println("--------------------------------").unwrap();

    // Total
    printer.println("Subtotal".bold().append("            $30.00")).unwrap();
    printer.println("Tax (10%)            $3.00").unwrap();
    printer.println("Total".bold().append("               $33.00".bold())).unwrap();

    printer.println("================================").unwrap();

    // Barcode
    printer.send(&SetJustification(Justification::Center)).unwrap();
    printer.send(&SetBarcodeHeight(60)).unwrap();
    printer.send(&SetHriPosition(HriPosition::Below)).unwrap();
    let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A00001").unwrap();
    printer.send(&barcode).unwrap();
    printer.send(&FeedLines(1)).unwrap();

    // QR Code
    let qr = PrintQrCode::new(b"RECEIPT:00001".to_vec())
        .unwrap()
        .with_module_size(4)
        .unwrap();
    printer.send(&qr).unwrap();

    // Footer
    printer.send(&FeedLines(1)).unwrap();
    printer.println("Thank you!").unwrap();

    printer.send(&FeedLines(3)).unwrap();
    printer.send(&CutPaper::partial()).unwrap();
    printer.flush().unwrap();
}
```

---

## Cargo.toml Feature

Add to Cargo.toml:

```toml
[features]
default = []
async = ["dep:tokio"]
rusb = ["dep:rusb"]
hardware-tests = []
```

---

## Running Hardware Tests

```bash
# Single test
BIXOLON_TEST_DEVICE=/dev/usb/lp0 cargo test --features hardware-tests hardware_basic_text -- --test-threads=1

# All hardware tests (use --test-threads=1 to prevent concurrent access)
BIXOLON_TEST_DEVICE=/dev/usb/lp0 cargo test --features hardware-tests -- --test-threads=1 hardware_
```

---

## Commit

```bash
git add tests/hardware.rs Cargo.toml
git commit -m "test: add hardware tests with physical printer"
```
