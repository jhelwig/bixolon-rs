# Bixolon SRP-350plus Rust Library Architecture

## Feature Overview

A Rust library for controlling Bixolon SRP-350plus USB thermal receipt printers. Provides complete ESC/POS command set support with ergonomic APIs for common operations, while maintaining full access to low-level commands.

## Goals

1. **Complete command coverage**: All 70+ documented ESC/POS commands available as composable structs
2. **Ergonomic high-level API**: Styled text, page builder, convenience methods
3. **Transport agnostic**: Core operates on `std::io::{Read, Write}` traits
4. **Both sync and async**: Feature-gated async support via tokio
5. **Excellent error messages**: miette diagnostics with source spans for encoding/validation errors
6. **Thoroughly tested**: Exact output verification in integration tests; hardware validation suite

## Architectural Approach

Three-layer architecture where each layer builds on the one below:

```
┌─────────────────────────────────────────────────────────┐
│  Ergonomic Layer                                        │
│  - StyledText (AST-based, composable)                   │
│  - PageBuilder (command queue)                          │
│  - Convenience only; composes Command layer             │
├─────────────────────────────────────────────────────────┤
│  Command Layer                                          │
│  - All ESC/POS commands as structs                      │
│  - #[repr(u8)] enums with doc comments                  │
│  - Complete, self-sufficient, no magic bytes            │
├─────────────────────────────────────────────────────────┤
│  Printer<W, R> / AsyncPrinter<W, R>                     │
│  - send(&impl Command)                                  │
│  - print/println for styled text                        │
│  - query for status (requires R: Read)                  │
│  - Works with any Read/Write implementor                │
└─────────────────────────────────────────────────────────┘
```

## Alternatives Considered

### State-tracking vs Stateless
- **Considered**: Library tracks current formatting state (bold, underline, etc.)
- **Rejected**: State can desync if printer reset externally; adds complexity
- **Chosen**: Stateless library with AST-based styled text that handles set/clear commands

### Builder pattern for text styling
- **Considered**: `printer.text("Hello").bold().print()` - fluent chaining
- **Rejected**: Misleading; ESC/POS requires formatting commands BEFORE text
- **Chosen**: StyledText AST with proper nesting; ergonomic traits like `"text".bold()`

### Single error crate
- **Considered**: Just thiserror, or just anyhow
- **Rejected**: Need both structured errors AND rich span diagnostics for encoding
- **Chosen**: thiserror for types + miette for diagnostic display

## Key Design Decisions

### 1. IO Abstraction
Core `Printer<W: Write, R = ()>` accepts any `Write` implementor. Reader is optional (defaults to `()`) for write-only use; `R: Read` enables status queries via separate impl block.

### 2. Command Representation
Every ESC/POS command is a struct implementing `Command` trait. All parameter values use `#[repr(u8)]` enums with explicit byte values and doc comments - no magic numbers hidden in match statements.

```rust
#[repr(u8)]
pub enum Justification {
    /// Left justification (default)
    Left = 0,
    /// Center justification
    Center = 1,
    /// Right justification
    Right = 2,
}
```

### 3. StyledText as AST
Styled text is a tree of `StyledNode` (Text or Styled with children). `StyleSet` holds `Option<T>` for each property. Effective style computed by folding all `StyleSet`s in stack - handles nested overrides correctly (e.g., underline → bold → no_underline → exit restores underline).

### 4. PageBuilder as Command Queue
Not text-specific; queues any `Command`. Convenience methods only for commands explicitly documented for page mode. Full page mode command set requires hardware verification before adding convenience methods.

### 5. Text Encoding
Both raw bytes and UTF-8 conversion APIs. Unrepresentable characters are errors with miette span diagnostics pointing at the exact character.

### 6. Feature Flags
- `async` - tokio async support
- `rusb` - USB transport via rusb crate
- `hardware-tests` - tests requiring physical printer

## Trade-offs

### Optimizing for:
- **Correctness**: Exact byte output, proper style nesting
- **Transparency**: No hidden magic; all values visible on enums
- **Ergonomics**: Styled text composition, convenience methods
- **Testability**: Mock IO for integration tests, exact output verification

### Accepting:
- **Verbosity for low-level use**: Raw command usage more verbose than high-level API
- **Async complexity**: Two parallel APIs (sync/async) with shared components
- **Page mode limitations**: Convenience methods limited to documented commands until hardware verified

## Constraints & Requirements

- Rust 1.92.0 stable, edition 2024
- No Cargo.lock (library crate)
- thiserror + miette for errors
- Hardware tests must fail (not skip) without `BIXOLON_TEST_DEVICE`
- NV memory writes limited to ~10/day (document in API)

## Technical Approach

### Module Structure

```
src/
├── lib.rs                    # Public API exports
├── error.rs                  # thiserror + miette error types
├── command/
│   ├── mod.rs               # Command trait, constants
│   ├── basic.rs             # HT, LF, FF, CR, CAN
│   ├── printer_control.rs   # Initialize, status queries
│   ├── character.rs         # Formatting commands
│   ├── codepage.rs          # Code page selection
│   ├── spacing.rs           # Positioning, margins
│   ├── page_mode.rs         # Page mode commands
│   ├── paper.rs             # Feed, cut
│   ├── image.rs             # Bit images, raster
│   ├── barcode.rs           # 1D barcodes
│   ├── symbol.rs            # QR, PDF417
│   └── status.rs            # Query commands
├── style/
│   ├── mod.rs               # StyledNode, StyleSet
│   ├── text.rs              # StyledText, Styleable trait
│   └── encoding.rs          # UTF-8 to code page conversion
├── page/
│   └── mod.rs               # PageBuilder
├── printer/
│   ├── mod.rs
│   ├── sync.rs              # Printer<W, R>
│   └── async.rs             # AsyncPrinter<W, R>
└── transport/
    └── usb.rs               # rusb convenience (feature-gated)
```

### Core Types

```rust
pub trait Command {
    fn encode(&self) -> Vec<u8>;
}

pub trait QueryCommand: Command {
    type Response;
    fn parse_response(&self, bytes: &[u8]) -> Result<Self::Response, ParseError>;
}

pub struct Printer<W: Write, R = ()> { ... }

// Query methods only when R: Read
impl<W: Write, R: Read> Printer<W, R> {
    pub fn query<Q: QueryCommand>(&mut self, q: &Q) -> Result<Q::Response, PrinterError>;
}

pub enum StyledNode {
    Text(String),
    Styled { style: StyleSet, children: Vec<StyledNode> },
}

pub struct StyleSet {
    pub emphasized: Option<bool>,
    pub underline: Option<UnderlineThickness>,
    pub size: Option<CharacterSize>,
    // ...
}

pub struct PageBuilder {
    area: Option<PrintArea>,
    direction: Option<PrintDirection>,
    commands: Vec<Box<dyn Command>>,
}
```

## Error Handling Strategy

Hierarchical errors with thiserror; miette diagnostics for user-facing errors:

```rust
#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("encoding error")]
    Encoding(#[from] EncodingError),
    // ...
}

#[derive(Debug, Error, Diagnostic)]
#[error("character not representable in {code_page:?}")]
pub struct EncodingError {
    #[source_code]
    pub src: String,
    #[label("this character cannot be encoded")]
    pub span: SourceSpan,
    pub code_page: CodePage,
}
```

## Testing Approach

1. **Unit tests**: Command encoding, validation, style resolution
2. **Integration tests**: End-to-end with mock `Read`/`Write`, exact byte verification
3. **Hardware tests** (`hardware-tests` feature):
   - Require `BIXOLON_TEST_DEVICE` env var; fail if not set
   - Print test name and expected output description
   - Manual verification of printed output

```rust
#[test]
fn styled_text_encodes_exact_bytes() {
    let mut output = Vec::new();
    let mut printer = Printer::new(&mut output);
    printer.println("Hello".bold())?;

    assert_eq!(output, vec![
        0x1B, b'E', 1,           // bold on
        b'H', b'e', b'l', b'l', b'o',
        0x1B, b'E', 0,           // bold off
        0x0A,                    // LF
    ]);
}
```
