# Bixolon SRP-350plus Rust Library Implementation Plan

> **For Claude:** When using subagent-driven-development, direct subagents to specific task files.
> **Full design details:** See [architecture.md](./architecture.md)

**Goal:** Create a Rust library for controlling Bixolon SRP-350plus USB thermal receipt printers with complete ESC/POS command support and ergonomic APIs.

**Architecture:** Three-layer design - Command layer (all ESC/POS commands as structs with `#[repr(u8)]` enums), Style layer (AST-based StyledText with stack-based resolution), and Printer layer (transport-agnostic `Printer<W, R>`). Ergonomic helpers compose commands, never bypass them.

**Tech Stack:** Rust 1.92.0, edition 2024, thiserror + miette for errors, tokio for async (optional), rusb for USB (optional)

## Rationale & Design Decisions

**Why three layers?**
- Command layer is complete and self-sufficient - users can always drop down to raw commands
- Ergonomic layer (StyledText, PageBuilder) provides convenience without hiding complexity
- Printer layer is transport-agnostic - works with any `Read`/`Write` implementor

**Why AST-based StyledText?**
- ESC/POS requires formatting commands BEFORE text they affect
- Nested styles need proper set/clear sequencing (e.g., underline → bold → no_underline → exit restores underline)
- Stack-based `StyleSet` resolution handles this correctly by recomputing effective style from all frames

**Why `#[repr(u8)]` with explicit values?**
- No magic bytes hidden in match statements
- Protocol values visible directly on enum variants
- Self-documenting with doc comments on each variant

**Why thiserror + miette?**
- thiserror for structured error types with `#[from]` conversions
- miette for rich diagnostics with source spans (pointing at exact character that can't be encoded)

**Why stateless?**
- Printer state can desync if reset externally
- AST-based styled text handles set/clear commands without tracking state

## Tasks

1. **[Project Foundation](./task-01-project-foundation.md)** - Cargo.toml, error types, Command trait, byte constants
2. **[Basic Commands](./task-02-basic-commands.md)** - HT, LF, FF, CR, CAN
3. **[Printer Control Commands](./task-03-printer-control.md)** - Initialize, peripheral selection
4. **[Character Formatting](./task-04-character-formatting.md)** - Emphasized, underline, character size, font, rotation, reverse
5. **[Code Pages](./task-05-code-pages.md)** - CodePage enum (40+ pages), international character sets
6. **[Spacing and Positioning](./task-06-spacing-positioning.md)** - Line spacing, tabs, margins, justification, absolute/relative positioning
7. **[Page Mode Commands](./task-07-page-mode-commands.md)** - Enter/exit page mode, print area, direction, vertical positioning
8. **[Paper and Cutting](./task-08-paper-cutting.md)** - Feed commands, cut modes
9. **[Barcode Commands](./task-09-barcode-commands.md)** - Height, width, HRI position, all barcode types with validation
10. **[2D Symbols](./task-10-2d-symbols.md)** - QR Code, PDF417 with full configuration
11. **[Image Commands](./task-11-image-commands.md)** - Bit images, raster images, NV images
12. **[Status and Query Commands](./task-12-status-query.md)** - Real-time status, printer ID, ASB
13. **[Macro Commands](./task-13-macro-commands.md)** - Macro definition and execution
14. **[StyleSet and Resolution](./task-14-styleset.md)** - StyleSet struct, stack-based resolution, transition emission
15. **[StyledNode AST](./task-15-stylednode.md)** - StyledNode enum, Styleable trait, text encoding with miette errors
16. **[PageBuilder](./task-16-pagebuilder.md)** - Command queue, convenience methods for documented page mode commands
17. **[Sync Printer](./task-17-sync-printer.md)** - Printer<W, R>, send/print/println/query methods
18. **[Async Printer](./task-18-async-printer.md)** - AsyncPrinter with tokio (feature-gated)
19. **[USB Transport](./task-19-usb-transport.md)** - rusb wrapper (feature-gated)
20. **[Public API and Docs](./task-20-public-api.md)** - lib.rs exports, documentation
21. **[Integration Tests](./task-21-integration-tests.md)** - Mock IO tests with exact byte verification
22. **[Hardware Tests](./task-22-hardware-tests.md)** - Real printer tests, fail without device
