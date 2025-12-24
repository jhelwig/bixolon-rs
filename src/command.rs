//! ESC/POS command definitions.
//!
//! All commands implement the [`Command`] trait for encoding to bytes.

pub mod barcode;
pub mod basic;
pub mod character;
pub mod codepage;
pub mod image;
pub mod macro_cmd;
pub mod page_mode;
pub mod paper;
pub mod printer_control;
pub mod spacing;
pub mod status;
pub mod symbol;

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
    /// Returns a [`StatusParseError`](crate::error::StatusParseError) if the response is malformed.
    fn parse_response(
        &self,
        bytes: &[u8],
    ) -> Result<Self::Response, crate::error::StatusParseError>;
}
