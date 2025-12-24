# Brainstorming Session: Bixolon SRP-350plus Rust Library

**Created:** 2025-12-22 (session start)
**Status:** Complete

## Quick Reference

### Requirements & Constraints
- Full ESC/POS control command set support (70+ commands from printer manual)
- Core library operates on `std::io::{Read, Write}` traits (not tied to specific USB/serial implementation)
- USB/serial transports as optional features providing convenience for obtaining Read/Write implementors
- Both sync and async API support (async via tokio feature flag)
- Proper Rust error handling with thiserror
- Rust 1.92.0 stable, edition 2024
- No Cargo.lock (library crate)
- Support streaming (standard) and page (buffered) print modes
- Unit testable design with meaningful tests
- Non-default integration test suite for real printer validation

### Alternatives Explored
- **USB Layer**: rusb vs nusb vs hidapi - User prefers library operate at Read/Write trait level, with USB/serial as optional convenience features
- **API Style**: Builder pattern vs command structs - User correctly noted builder patterns misleading for ESC/POS where formatting must precede text; chose explicit command structs with convenience methods

### Key Decisions
1. **IO Abstraction**: Core `Printer<W: Write, R: Read>` - transport-agnostic
2. **Command Representation**: Explicit command structs implementing `Command` trait (not builders)
3. **Convenience Methods**: `.text()`, `.qr_code()`, etc. that use command structs internally
4. **Feature Flags**: `async`, `usb`, `serial`, `integration-tests`
5. **State Management**: Stateless library - no internal tracking of printer modes
6. **Styled Text**: Ergonomic styling via extension traits/wrapper (like ANSI color crates) that emit set+clear commands around text
7. **Line Feeds**: Explicit - either `print`/`println` naming or `.line_feed()` method on styled text
8. **Error Handling**: thiserror for error types + miette for diagnostic display (source spans for encoding errors)
9. **Text Encoding**: Option D - both raw bytes and UTF-8 conversion APIs; unrepresentable characters are errors with miette span diagnostics
10. **Page Mode**: Builder pattern - `PageBuilder` collects commands, `printer.print_page(&page)` executes all at once
11. **Ergonomic helpers are convenience only**: Everything doable via command layer; no magic bytes required
12. **Feature naming**: Named after crate (`rusb` not `usb`); drop serial (no hardware to test)
13. **Test naming**: `hardware-tests` for physical printer; integration tests use mock IO
14. **Reader generic**: `R = ()` default for write-only; `R: Read` enables query methods via separate impl block
15. **Enum values explicit**: Use `#[repr(u8)]` with values on variants, not hidden in match statements
16. **Doc comments on variants**: All enum variants have doc comments explaining meaning, constraints, use cases
17. **StyledText is AST-based**: Tree of `StyledNode` (Text or Styled with children), not flat strings
18. **StyleSet for style properties**: `StyleSet` holds `Option<T>` for each property; `None` = inherit/default
19. **Stack-based resolution**: Effective style computed by folding all `StyleSet`s in stack; handles nested overrides correctly
20. **Transition emission**: Compare before/after `StyleSet` to emit only changed commands; restoring previous style recomputes from remaining stack
21. **PageBuilder is a command queue**: Not text-specific; queues any `Command`, convenience methods just construct and append
22. **PageBuilder convenience methods match documented commands only**: Only expose what manual explicitly documents for page mode (ESC W, ESC T, GS $, etc.)
23. **Page mode command set requires hardware verification**: Full set of commands that work in page mode must be verified against actual hardware before adding convenience methods
24. **Error types use thiserror**: `PrinterError` as top-level with `#[from]` for sub-errors
25. **Encoding/validation errors use miette**: `#[derive(Diagnostic)]` with `#[source_code]`, `#[label]`, `#[help]` for rich span-based diagnostics
26. **Integration tests verify exact output**: No "contains" or "starts_with"; assert exact byte sequences for deterministic commands
27. **Hardware tests fail without printer**: No skipping; `BIXOLON_TEST_DEVICE` must be set or test fails
28. **Hardware tests print identification**: Each test prints its name and expected output description for manual verification
29. **Async API mirrors sync**: `AsyncPrinter<W, R>` with same methods as `Printer<W, R>` but async
30. **Shared components**: Command trait, command structs, StyledNode, StyleSet, PageBuilder, errors - all shared between sync/async

### Current Design State
- Module structure drafted (command/, encoding/, printer/, transport/)
- Command trait defined with `encode()` and `encoded_len()`
- QueryCommand trait for bidirectional communication
- Printer<W, R> struct with send/query/flush + convenience methods
- Error hierarchy with thiserror (PrinterError, ValidationError, BarcodeError, etc.)
- CodePage enum covering 40+ code pages
- Barcode validation logic drafted

### Open Questions
(All resolved - see Key Decisions)

---

## Chronological Log

### [Phase 1] Initial Understanding
- Downloaded and analyzed three printer manuals:
  - Control commands manual: 70+ ESC/POS commands extracted
  - User manual: USB/Serial interfaces, 4KB buffer, 180 dpi, paper specs
  - Code pages manual: 40+ code pages, international character sets
- User clarified: Library should operate on Read/Write traits, not tied to specific USB implementation
- User clarified: Both sync and async support needed
- User clarified: Command structs preferred over builder patterns (formatting must precede text)

### [Phase 1] Requirements Gathered
- Streaming mode: Commands print immediately
- Page mode: Buffer commands, print with FF
- Barcodes: UPC-A/E, JAN13/8, CODE39, ITF, CODABAR, CODE93, CODE128
- 2D codes: PDF417, QR Code with full configuration
- Status queries: Paper sensor, drawer, offline status
- NV memory: Limited writes (~10/day) - should document this constraint

### [Phase 2] Design Decisions
- Stateless library with ergonomic styled text helpers (like ANSI color crates)
- Explicit line feeds via print/println or .line_feed()
- thiserror + miette for error handling (miette for encoding span diagnostics)
- Both raw bytes and UTF-8 conversion APIs for text
- PageBuilder pattern for page mode (collect then execute)

### [Phase 3] Design Presentation
- Core architecture validated: Command layer → Ergonomic layer → Printer
- Command layer uses `#[repr(u8)]` enums with doc comments, no hidden magic bytes
- StyledText is AST-based with stack-based style resolution for proper nesting
- PageBuilder is command queue, not text-specific; convenience methods match documented page mode commands only
- Error handling: thiserror + miette for span diagnostics on encoding/validation errors
- Testing: exact output verification for integration tests; hardware tests fail without printer and print test identification
- Async API mirrors sync, shares all non-IO components

### [Phase 5] Architecture Documentation
- Created architecture.md at docs/plans/2025-12-22-bixolon-printer-library/architecture.md
- Documents: feature overview, goals, architectural approach, alternatives considered, key decisions, trade-offs, constraints, technical approach, error handling, testing
