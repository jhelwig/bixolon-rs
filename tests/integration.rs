//! Integration tests for bixolon library.
//!
//! These tests verify complete byte sequences, not partial matches.

use bixolon::command::barcode::*;
use bixolon::command::basic::*;
use bixolon::command::character::*;
use bixolon::command::page_mode::*;
use bixolon::command::paper::*;
use bixolon::command::symbol::*;
use bixolon::page::PageBuilder;
use bixolon::printer::Printer;
use bixolon::style::text::Styleable;

/// Test helper macro to capture printer output.
///
/// Accepts closures returning `Result<T, PrinterError>` for any `T`.
/// The return value is discarded; only the bytes written to the printer are captured.
macro_rules! capture_output {
    // Block form: |p| { ... } - needs closure wrapper to support `?`
    (|$p:ident| { $($body:tt)* }) => {{
        let buf = Vec::new();
        let mut printer = Printer::new(buf);
        (|$p: &mut Printer<Vec<u8>, ()>| { $($body)* })(&mut printer)
            .map(|_| ())
            .expect("printer operation failed");
        printer.flush().expect("flush failed");
        let (inner, _) = printer.into_inner();
        inner
    }};
    // Expression form: |p| expr - direct evaluation
    (|$p:ident| $body:expr) => {{
        let buf = Vec::new();
        let mut $p = Printer::new(buf);
        ($body).map(|_| ()).expect("printer operation failed");
        $p.flush().expect("flush failed");
        let (inner, _) = $p.into_inner();
        inner
    }};
}

// ============================================================================
// Basic Commands
// ============================================================================

#[test]
fn test_initialize() {
    let output = capture_output!(|p| p.send(&bixolon::command::printer_control::Initialize));
    assert_eq!(output, vec![0x1B, b'@']);
}

#[test]
fn test_line_feed() {
    let output = capture_output!(|p| p.send(&LineFeed));
    assert_eq!(output, vec![0x0A]);
}

#[test]
fn test_form_feed() {
    let output = capture_output!(|p| p.send(&FormFeed));
    assert_eq!(output, vec![0x0C]);
}

#[test]
fn test_horizontal_tab() {
    let output = capture_output!(|p| p.send(&HorizontalTab));
    assert_eq!(output, vec![0x09]);
}

// ============================================================================
// Character Formatting
// ============================================================================

#[test]
fn test_set_emphasized_on() {
    let output = capture_output!(|p| p.send(&SetEmphasized(true)));
    assert_eq!(output, vec![0x1B, b'E', 1]);
}

#[test]
fn test_set_emphasized_off() {
    let output = capture_output!(|p| p.send(&SetEmphasized(false)));
    assert_eq!(output, vec![0x1B, b'E', 0]);
}

#[test]
fn test_underline_single() {
    let output = capture_output!(|p| p.send(&SetUnderline(UnderlineThickness::OneDot)));
    assert_eq!(output, vec![0x1B, b'-', 1]);
}

#[test]
fn test_underline_double() {
    let output = capture_output!(|p| p.send(&SetUnderline(UnderlineThickness::TwoDot)));
    assert_eq!(output, vec![0x1B, b'-', 2]);
}

#[test]
fn test_justification_center() {
    let output = capture_output!(|p| p.send(&SetJustification(Justification::Center)));
    assert_eq!(output, vec![0x1B, b'a', 1]);
}

// ============================================================================
// Styled Text
// ============================================================================

#[test]
fn test_plain_text_println() {
    let output = capture_output!(|p| p.println("Hello"));
    // Plain text + LF
    assert_eq!(output, b"Hello\x0A".to_vec());
}

#[test]
fn test_bold_text_println() {
    let output = capture_output!(|p| p.println("Hello".bold()));

    let expected = vec![
        0x1B, b'E', 1, // ESC E 1 (bold on)
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        0x1B, b'E', 0,    // ESC E 0 (bold off)
        0x0A, // LF
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_underlined_text_println() {
    let output = capture_output!(|p| p.println("Hello".underlined()));

    let expected = vec![
        0x1B, b'-', 1, // ESC - 1 (underline on)
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        0x1B, b'-', 0,    // ESC - 0 (underline off)
        0x0A, // LF
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_nested_bold_underline() {
    // Bold "outer " then underlined "inner" as siblings
    let output = capture_output!(|p| p.println("outer ".bold().append("inner".underlined())));

    // Exact expected output:
    // Bold on, "outer ", bold off, underline on, "inner", underline off, LF
    let expected = vec![
        0x1B, b'E', 1, // ESC E 1 (bold on)
        b'o', b'u', b't', b'e', b'r', b' ', // "outer "
        0x1B, b'E', 0, // ESC E 0 (bold off)
        0x1B, b'-', 1, // ESC - 1 (underline on)
        b'i', b'n', b'n', b'e', b'r', // "inner"
        0x1B, b'-', 0,    // ESC - 0 (underline off)
        0x0A, // LF
    ];
    assert_eq!(output, expected);
}

#[test]
fn test_sibling_styles_isolated() {
    // "A" bold, "B" underlined - styles should not leak between siblings
    let output = capture_output!(|p| p.println("A".bold().append("B".underlined())));

    // Exact expected output:
    // Bold on, "A", bold off, underline on, "B", underline off, LF
    let expected = vec![
        0x1B, b'E', 1,    // ESC E 1 (bold on)
        b'A', // "A"
        0x1B, b'E', 0, // ESC E 0 (bold off)
        0x1B, b'-', 1,    // ESC - 1 (underline on)
        b'B', // "B"
        0x1B, b'-', 0,    // ESC - 0 (underline off)
        0x0A, // LF
    ];
    assert_eq!(output, expected);
}

// ============================================================================
// Paper Commands
// ============================================================================

#[test]
fn test_cut_full() {
    let output = capture_output!(|p| p.send(&CutPaper::full()));
    assert_eq!(output, vec![0x1D, b'V', 0]);
}

#[test]
fn test_cut_partial() {
    let output = capture_output!(|p| p.send(&CutPaper::partial()));
    assert_eq!(output, vec![0x1D, b'V', 1]);
}

#[test]
fn test_cut_feed_and_partial() {
    let output = capture_output!(|p| p.send(&CutPaper::feed_and_partial(5)));
    assert_eq!(output, vec![0x1D, b'V', 66, 5]);
}

#[test]
fn test_feed_lines() {
    let output = capture_output!(|p| p.send(&FeedLines(3)));
    assert_eq!(output, vec![0x1B, b'd', 3]);
}

// ============================================================================
// Barcodes
// ============================================================================

#[test]
fn test_barcode_height() {
    let output = capture_output!(|p| p.send(&SetBarcodeHeight(100)));
    assert_eq!(output, vec![0x1D, b'h', 100]);
}

#[test]
fn test_barcode_width() {
    let output = capture_output!(|p| p.send(&SetBarcodeWidth(BarcodeWidth::Wide)));
    assert_eq!(output, vec![0x1D, b'w', 5]);
}

#[test]
fn test_hri_position_below() {
    let output = capture_output!(|p| p.send(&SetHriPosition(HriPosition::Below)));
    assert_eq!(output, vec![0x1D, b'H', 2]);
}

#[test]
fn test_print_barcode_code128() {
    let barcode = PrintBarcode::new(BarcodeSystem::Code128, b"{A123").unwrap();
    let output = capture_output!(|p| p.send(&barcode));

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
    let output = capture_output!(|p| p.send(&qr));

    // Exact expected output for QR code with "Hello" data
    let expected = vec![
        // Function 165: Select model (Model 2)
        0x1D, b'(', b'k', 0x04, 0x00, b'1', b'A', b'2', 0x00,
        // Function 167: Set module size (3)
        0x1D, b'(', b'k', 0x03, 0x00, b'1', b'C', 0x03,
        // Function 169: Set error correction (L=48)
        0x1D, b'(', b'k', 0x03, 0x00, b'1', b'E', b'0',
        // Function 180: Store data "Hello"
        0x1D, b'(', b'k', 0x08, 0x00, b'1', b'P', b'0', b'H', b'e', b'l', b'l', b'o',
        // Function 181: Print symbol
        0x1D, b'(', b'k', 0x03, 0x00, b'1', b'Q', b'0',
    ];
    assert_eq!(output, expected);
}

// ============================================================================
// Page Mode
// ============================================================================

#[test]
fn test_page_builder_empty() {
    let page = PageBuilder::new().build();

    // Exact: ESC L, FF
    let expected = vec![0x1B, b'L', 0x0C];
    assert_eq!(page, expected);
}

#[test]
fn test_page_builder_with_text() {
    let page = PageBuilder::new().text("Test").build();

    // Exact: ESC L, "Test", FF
    let expected = vec![0x1B, b'L', b'T', b'e', b's', b't', 0x0C];
    assert_eq!(page, expected);
}

#[test]
fn test_page_builder_with_area() {
    let page = PageBuilder::new()
        .area(PrintArea {
            x: 0,
            y: 0,
            width: 512,
            height: 400,
        })
        .build();

    // Exact: ESC L, ESC W (area: x=0, y=0, w=512, h=400), FF
    // Area params are 8 bytes: xL xH yL yH wL wH hL hH
    // 512 = 0x0200, 400 = 0x0190
    let expected = vec![
        0x1B, b'L', // Enter page mode
        0x1B, b'W', // Set print area
        0x00, 0x00, // x = 0
        0x00, 0x00, // y = 0
        0x00, 0x02, // width = 512 (little endian)
        0x90, 0x01, // height = 400 (little endian)
        0x0C, // FF
    ];
    assert_eq!(page, expected);
}

#[test]
fn test_page_builder_with_direction() {
    let page = PageBuilder::new().direction(PrintDirection::BottomToTop).build();

    // Exact: ESC L, ESC T 1, FF
    let expected = vec![0x1B, b'L', 0x1B, b'T', 0x01, 0x0C];
    assert_eq!(page, expected);
}

#[test]
fn test_page_builder_complete() {
    let page = PageBuilder::new()
        .area(PrintArea::default_80mm())
        .direction(PrintDirection::LeftToRight)
        .vertical_position(100)
        .text_line("Header".bold())
        .build();

    // Exact expected output
    // PrintArea::default_80mm() = { x: 0, y: 0, width: 512, height: 1662 }
    // 512 = 0x0200, 1662 = 0x067E
    let expected = vec![
        0x1B, b'L', // Enter page mode
        0x1B, b'W', // Set print area
        0x00, 0x00, // x = 0
        0x00, 0x00, // y = 0
        0x00, 0x02, // width = 512
        0x7E, 0x06, // height = 1662
        0x1B, b'T', 0x00, // Direction: LeftToRight
        0x1D, b'$', 0x64, 0x00, // Vertical position = 100
        0x1B, b'E', 0x01, // Bold on
        b'H', b'e', b'a', b'd', b'e', b'r', // "Header"
        0x1B, b'E', 0x00, // Bold off
        0x0A, // LF (from text_line)
        0x0C, // FF (page end)
    ];
    assert_eq!(page, expected);
}

// ============================================================================
// Full Receipt Test
// ============================================================================

#[test]
fn test_complete_receipt() {
    let output = capture_output!(|p| {
        p.send(&bixolon::command::printer_control::Initialize)?;
        p.println("RECEIPT".bold())?;
        p.println("--------")?;
        p.println("Item 1    $10.00")?;
        p.println("Item 2    $15.00")?;
        p.println("--------")?;
        p.println("Total".bold().append("     $25.00".bold()))?;
        p.send(&FeedLines(3))?;
        p.send(&CutPaper::partial())?;
        Ok::<(), bixolon::error::PrinterError>(())
    });

    // Exact expected output
    let expected = vec![
        // Initialize
        0x1B, b'@', // "RECEIPT" bold + LF
        0x1B, b'E', 0x01, b'R', b'E', b'C', b'E', b'I', b'P', b'T', 0x1B, b'E', 0x00, 0x0A,
        // "--------" + LF
        b'-', b'-', b'-', b'-', b'-', b'-', b'-', b'-', 0x0A, // "Item 1    $10.00" + LF
        b'I', b't', b'e', b'm', b' ', b'1', b' ', b' ', b' ', b' ', b'$', b'1', b'0', b'.', b'0',
        b'0', 0x0A, // "Item 2    $15.00" + LF
        b'I', b't', b'e', b'm', b' ', b'2', b' ', b' ', b' ', b' ', b'$', b'1', b'5', b'.', b'0',
        b'0', 0x0A, // "--------" + LF
        b'-', b'-', b'-', b'-', b'-', b'-', b'-', b'-', 0x0A,
        // "Total" bold, "     $25.00" bold (siblings) + LF
        0x1B, b'E', 0x01, b'T', b'o', b't', b'a', b'l', 0x1B, b'E', 0x00, 0x1B, b'E', 0x01, b' ',
        b' ', b' ', b' ', b' ', b'$', b'2', b'5', b'.', b'0', b'0', 0x1B, b'E', 0x00, 0x0A,
        // Feed 3 lines
        0x1B, b'd', 0x03, // Partial cut
        0x1D, b'V', 0x01,
    ];
    assert_eq!(output, expected);
}
