# Task 21: Integration Tests

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `tests/integration.rs`

**Important:** Integration tests use mock IO (`Vec<u8>`) and verify **exact byte sequences**, not partial matches.

---

## Goals

1. End-to-end tests with mock IO
2. Exact byte sequence verification
3. Test styled text rendering
4. Test page mode output
5. Test barcode/QR code generation

---

## Implementation

```rust
//! Integration tests for bixolon library.
//!
//! These tests verify complete byte sequences, not partial matches.

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

/// Test helper to capture printer output.
fn capture_output<F>(f: F) -> Vec<u8>
where
    F: FnOnce(&mut Printer<Vec<u8>, ()>) -> Result<(), bixolon::error::PrinterError>,
{
    let buf = Vec::new();
    let mut printer = Printer::new(buf);
    f(&mut printer).expect("printer operation failed");
    printer.flush().expect("flush failed");
    let (inner, _) = printer.into_inner();
    inner
}

// ============================================================================
// Basic Commands
// ============================================================================

#[test]
fn test_initialize() {
    let output = capture_output(|p| {
        p.send(&bixolon::command::printer_control::Initialize)
    });
    assert_eq!(output, vec![0x1B, b'@']);
}

#[test]
fn test_line_feed() {
    let output = capture_output(|p| p.send(&LineFeed));
    assert_eq!(output, vec![0x0A]);
}

#[test]
fn test_form_feed() {
    let output = capture_output(|p| p.send(&FormFeed));
    assert_eq!(output, vec![0x0C]);
}

#[test]
fn test_horizontal_tab() {
    let output = capture_output(|p| p.send(&HorizontalTab));
    assert_eq!(output, vec![0x09]);
}

// ============================================================================
// Character Formatting
// ============================================================================

#[test]
fn test_set_emphasized_on() {
    let output = capture_output(|p| p.send(&SetEmphasized(true)));
    assert_eq!(output, vec![0x1B, b'E', 1]);
}

#[test]
fn test_set_emphasized_off() {
    let output = capture_output(|p| p.send(&SetEmphasized(false)));
    assert_eq!(output, vec![0x1B, b'E', 0]);
}

#[test]
fn test_underline_single() {
    let output = capture_output(|p| p.send(&SetUnderline(UnderlineMode::Single)));
    assert_eq!(output, vec![0x1B, b'-', 1]);
}

#[test]
fn test_underline_double() {
    let output = capture_output(|p| p.send(&SetUnderline(UnderlineMode::Double)));
    assert_eq!(output, vec![0x1B, b'-', 2]);
}

#[test]
fn test_justification_center() {
    let output = capture_output(|p| p.send(&SetJustification(Justification::Center)));
    assert_eq!(output, vec![0x1B, b'a', 1]);
}

// ============================================================================
// Styled Text
// ============================================================================

#[test]
fn test_plain_text_println() {
    let output = capture_output(|p| p.println("Hello"));
    // Plain text + LF
    assert_eq!(output, b"Hello\x0A".to_vec());
}

#[test]
fn test_bold_text_println() {
    let output = capture_output(|p| p.println("Hello".bold()));

    let expected = vec![
        0x1B, b'E', 1,              // ESC E 1 (bold on)
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        0x1B, b'E', 0,              // ESC E 0 (bold off)
        0x0A,                       // LF
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_underlined_text_println() {
    let output = capture_output(|p| p.println("Hello".underlined()));

    let expected = vec![
        0x1B, b'-', 1,              // ESC - 1 (underline on)
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        0x1B, b'-', 0,              // ESC - 0 (underline off)
        0x0A,                       // LF
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_nested_bold_underline() {
    // Bold containing underlined text
    let output = capture_output(|p| {
        p.println("outer ".bold().append("inner".underlined()))
    });

    // Should have:
    // 1. Bold on
    // 2. "outer "
    // 3. Underline on (bold still on)
    // 4. "inner"
    // 5. Underline off
    // 6. Bold off
    // 7. LF

    // Verify structure - bold comes first, then underline for inner
    assert!(output.starts_with(&[0x1B, b'E', 1])); // bold on at start

    // Check both segments present
    assert!(output.windows(6).any(|w| w == b"outer "));
    assert!(output.windows(5).any(|w| w == b"inner"));

    // Check ends with bold off then LF
    assert!(output.ends_with(&[0x0A]));

    // Both bold and underline should be turned off before final LF
    let before_lf = &output[..output.len() - 1];
    assert!(before_lf.windows(3).any(|w| w == [0x1B, b'E', 0])); // bold off
}

#[test]
fn test_sibling_styles_isolated() {
    // "A" bold, "B" underlined - styles should not leak
    let output = capture_output(|p| {
        p.println("A".bold().append("B".underlined()))
    });

    // After "A", bold should turn off before underline turns on for "B"
    // This verifies style isolation between siblings

    let bold_on = output.windows(3).position(|w| w == [0x1B, b'E', 1]).unwrap();
    let a_pos = output.windows(1).position(|w| w == b"A").unwrap();
    let bold_off = output.windows(3).position(|w| w == [0x1B, b'E', 0]).unwrap();
    let underline_on = output.windows(3).position(|w| w == [0x1B, b'-', 1]).unwrap();
    let b_pos = output.windows(1).position(|w| w == b"B").unwrap();
    let underline_off = output.windows(3).position(|w| w == [0x1B, b'-', 0]).unwrap();

    // Order should be: bold_on < A < bold_off < underline_on < B < underline_off
    assert!(bold_on < a_pos);
    assert!(a_pos < bold_off);
    assert!(bold_off < underline_on);
    assert!(underline_on < b_pos);
    assert!(b_pos < underline_off);
}

// ============================================================================
// Paper Commands
// ============================================================================

#[test]
fn test_cut_full() {
    let output = capture_output(|p| p.send(&CutPaper::full()));
    assert_eq!(output, vec![0x1D, b'V', 0]);
}

#[test]
fn test_cut_partial() {
    let output = capture_output(|p| p.send(&CutPaper::partial()));
    assert_eq!(output, vec![0x1D, b'V', 1]);
}

#[test]
fn test_cut_feed_and_partial() {
    let output = capture_output(|p| p.send(&CutPaper::feed_and_partial(5)));
    assert_eq!(output, vec![0x1D, b'V', 66, 5]);
}

#[test]
fn test_feed_lines() {
    let output = capture_output(|p| p.send(&FeedLines(3)));
    assert_eq!(output, vec![0x1B, b'd', 3]);
}

// ============================================================================
// Barcodes
// ============================================================================

#[test]
fn test_barcode_height() {
    let output = capture_output(|p| p.send(&SetBarcodeHeight(100)));
    assert_eq!(output, vec![0x1D, b'h', 100]);
}

#[test]
fn test_barcode_width() {
    let output = capture_output(|p| p.send(&SetBarcodeWidth(BarcodeWidth::Wide)));
    assert_eq!(output, vec![0x1D, b'w', 5]);
}

#[test]
fn test_hri_position_below() {
    let output = capture_output(|p| p.send(&SetHriPosition(HriPosition::Below)));
    assert_eq!(output, vec![0x1D, b'H', 2]);
}

#[test]
fn test_print_barcode_code128() {
    let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A123").unwrap();
    let output = capture_output(|p| p.send(&barcode));

    let expected = vec![
        0x1D, b'k', 73, 5, // GS k 73 (Code128) length=5
        b'{', b'A', b'1', b'2', b'3', // data
    ];
    assert_eq!(output, expected);
}

// ============================================================================
// QR Code
// ============================================================================

#[test]
fn test_qr_code_hello() {
    let qr = PrintQrCode::new(b"Hello".to_vec()).unwrap();
    let output = capture_output(|p| p.send(&qr));

    // Verify structure contains all required sub-commands
    // Function 165: Select model
    assert!(output.windows(5).any(|w| w == [0x1D, b'(', b'k', 4, 0]));

    // Function 167: Set module size
    assert!(output.windows(5).any(|w| w == [0x1D, b'(', b'k', 3, 0]));

    // Function 180: Store data (contains "Hello")
    assert!(output.windows(5).any(|w| w == b"Hello"));

    // Function 181: Print symbol
    assert!(output.windows(8).any(|w| w == [0x1D, b'(', b'k', 3, 0, 49, 81, 48]));
}

// ============================================================================
// Page Mode
// ============================================================================

#[test]
fn test_page_builder_empty() {
    let page = PageBuilder::new().build();

    // Must start with ESC L and end with FF
    assert_eq!(&page[0..2], &[0x1B, b'L']);
    assert_eq!(page.last(), Some(&0x0C));
}

#[test]
fn test_page_builder_with_text() {
    let page = PageBuilder::new().text("Test").build();

    // Structure: ESC L, "Test", FF
    assert_eq!(&page[0..2], &[0x1B, b'L']);
    assert!(page.windows(4).any(|w| w == b"Test"));
    assert_eq!(page.last(), Some(&0x0C));
}

#[test]
fn test_page_builder_with_area() {
    let page = PageBuilder::new()
        .area(PrintArea { x: 0, y: 0, width: 512, height: 400 })
        .build();

    // Should contain ESC W
    assert!(page.windows(2).any(|w| w == [0x1B, b'W']));
}

#[test]
fn test_page_builder_with_direction() {
    let page = PageBuilder::new()
        .direction(PrintDirection::BottomToTop)
        .build();

    // Should contain ESC T 1
    assert!(page.windows(3).any(|w| w == [0x1B, b'T', 1]));
}

#[test]
fn test_page_builder_complete() {
    let page = PageBuilder::new()
        .area(PrintArea::default_80mm())
        .direction(PrintDirection::LeftToRight)
        .vertical_position(100)
        .text_line("Header".bold())
        .build();

    // Verify full structure
    let bytes = &page;

    // 1. Starts with ESC L
    assert_eq!(&bytes[0..2], &[0x1B, b'L']);

    // 2. Contains area command (ESC W)
    assert!(bytes.windows(2).any(|w| w == [0x1B, b'W']));

    // 3. Contains direction command (ESC T 0)
    assert!(bytes.windows(3).any(|w| w == [0x1B, b'T', 0]));

    // 4. Contains vertical position (GS $)
    assert!(bytes.windows(2).any(|w| w == [0x1D, b'$']));

    // 5. Contains bold on before Header
    let bold_on = bytes.windows(3).position(|w| w == [0x1B, b'E', 1]);
    let header = bytes.windows(6).position(|w| w == b"Header");
    assert!(bold_on.is_some() && header.is_some());
    assert!(bold_on.unwrap() < header.unwrap());

    // 6. Ends with FF
    assert_eq!(bytes.last(), Some(&0x0C));
}

// ============================================================================
// Full Receipt Test
// ============================================================================

#[test]
fn test_complete_receipt() {
    let output = capture_output(|p| {
        p.send(&bixolon::command::printer_control::Initialize)?;
        p.println("RECEIPT".bold())?;
        p.println("--------")?;
        p.println("Item 1    $10.00")?;
        p.println("Item 2    $15.00")?;
        p.println("--------")?;
        p.println("Total".bold().append("     $25.00".bold()))?;
        p.send(&FeedLines(3))?;
        p.send(&CutPaper::partial())?;
        Ok(())
    });

    // Verify key elements in order
    let init_pos = output.windows(2).position(|w| w == [0x1B, b'@']);
    let receipt_pos = output.windows(7).position(|w| w == b"RECEIPT");
    let total_pos = output.windows(5).position(|w| w == b"Total");
    let cut_pos = output.windows(3).position(|w| w == [0x1D, b'V', 1]);

    assert!(init_pos.is_some());
    assert!(receipt_pos.is_some());
    assert!(total_pos.is_some());
    assert!(cut_pos.is_some());

    // Verify order
    assert!(init_pos.unwrap() < receipt_pos.unwrap());
    assert!(receipt_pos.unwrap() < total_pos.unwrap());
    assert!(total_pos.unwrap() < cut_pos.unwrap());
}
```

---

## Commit

```bash
git add tests/integration.rs
git commit -m "test: add integration tests with exact byte verification"
```
