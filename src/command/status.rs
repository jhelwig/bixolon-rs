//! Status and query commands.
//!
//! These commands query printer state and require reading a response.

use super::{Command, DLE, GS, QueryCommand};
use crate::error::StatusParseError;

/// Real-time status type for DLE EOT.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    /// Printer status (online, drawer, paper).
    Printer = 1,
    /// Offline status (cover, paper feed, error).
    Offline = 2,
    /// Error status (recoverable, auto-cutter, unrecoverable).
    Error = 3,
    /// Paper roll sensor status.
    PaperRoll = 4,
}

/// Real-time status transmission.
///
/// ESC/POS: `DLE EOT n` (0x10 0x04 n)
///
/// This is a real-time command that can be sent even during printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransmitStatus(pub StatusType);

impl Command for TransmitStatus {
    fn encode(&self) -> Vec<u8> {
        vec![DLE, 0x04, self.0 as u8]
    }
}

/// Printer status response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrinterStatus {
    /// Drawer kick-out connector pin 3 status.
    pub drawer_open: bool,
    /// Printer is online.
    pub online: bool,
    /// Paper feed switch is being pressed.
    pub feed_button_pressed: bool,
    /// Paper is present (not at end).
    pub paper_present: bool,
}

impl PrinterStatus {
    fn parse(byte: u8) -> Self {
        Self {
            drawer_open: byte & 0x04 != 0,
            online: byte & 0x08 == 0,
            feed_button_pressed: byte & 0x20 != 0,
            paper_present: byte & 0x60 != 0x60,
        }
    }
}

/// Offline status response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineStatus {
    /// Cover is open.
    pub cover_open: bool,
    /// Paper is being fed by feed button.
    pub paper_feeding: bool,
    /// Recoverable error occurred.
    pub recoverable_error: bool,
    /// Auto-cutter error occurred.
    pub cutter_error: bool,
}

impl OfflineStatus {
    fn parse(byte: u8) -> Self {
        Self {
            cover_open: byte & 0x04 != 0,
            paper_feeding: byte & 0x08 != 0,
            recoverable_error: byte & 0x20 != 0,
            cutter_error: byte & 0x40 != 0,
        }
    }
}

/// Error status response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorStatus {
    /// Recoverable error occurred.
    pub recoverable_error: bool,
    /// Auto-cutter error occurred.
    pub cutter_error: bool,
    /// Unrecoverable error occurred.
    pub unrecoverable_error: bool,
}

impl ErrorStatus {
    fn parse(byte: u8) -> Self {
        Self {
            recoverable_error: byte & 0x04 != 0,
            cutter_error: byte & 0x08 != 0,
            unrecoverable_error: byte & 0x20 != 0,
        }
    }
}

/// Paper roll sensor status response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaperRollStatus {
    /// Paper near end detected.
    pub paper_near_end: bool,
    /// Paper end detected.
    pub paper_end: bool,
}

impl PaperRollStatus {
    fn parse(byte: u8) -> Self {
        Self {
            paper_near_end: byte & 0x0C != 0,
            paper_end: byte & 0x60 != 0,
        }
    }
}

/// Combined status response from TransmitStatus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusResponse {
    /// Printer status.
    Printer(PrinterStatus),
    /// Offline status.
    Offline(OfflineStatus),
    /// Error status.
    Error(ErrorStatus),
    /// Paper roll sensor status.
    PaperRoll(PaperRollStatus),
}

impl QueryCommand for TransmitStatus {
    type Response = StatusResponse;

    fn parse_response(&self, bytes: &[u8]) -> Result<Self::Response, StatusParseError> {
        if bytes.is_empty() {
            return Err(StatusParseError::EmptyResponse);
        }
        let byte = bytes[0];

        Ok(match self.0 {
            StatusType::Printer => StatusResponse::Printer(PrinterStatus::parse(byte)),
            StatusType::Offline => StatusResponse::Offline(OfflineStatus::parse(byte)),
            StatusType::Error => StatusResponse::Error(ErrorStatus::parse(byte)),
            StatusType::PaperRoll => StatusResponse::PaperRoll(PaperRollStatus::parse(byte)),
        })
    }
}

/// ASB (Automatic Status Back) enable flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AsbFlags {
    /// Enable drawer kick-out connector status.
    pub drawer: bool,
    /// Enable online/offline status.
    pub online_offline: bool,
    /// Enable error status.
    pub error: bool,
    /// Enable paper roll sensor status.
    pub paper_roll: bool,
}

impl AsbFlags {
    /// All status types enabled.
    pub const fn all() -> Self {
        Self {
            drawer: true,
            online_offline: true,
            error: true,
            paper_roll: true,
        }
    }

    /// No status types enabled.
    pub const fn none() -> Self {
        Self {
            drawer: false,
            online_offline: false,
            error: false,
            paper_roll: false,
        }
    }

    fn to_byte(self) -> u8 {
        let mut n = 0u8;
        if self.drawer {
            n |= 0x01;
        }
        if self.online_offline {
            n |= 0x02;
        }
        if self.error {
            n |= 0x04;
        }
        if self.paper_roll {
            n |= 0x08;
        }
        n
    }
}

/// Enable/disable Automatic Status Back (ASB).
///
/// ESC/POS: `GS a n` (0x1D 0x61 n)
///
/// When enabled, the printer automatically transmits status when
/// status changes occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EnableAsb(pub AsbFlags);

impl Command for EnableAsb {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'a', self.0.to_byte()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_type_values() {
        assert_eq!(StatusType::Printer as u8, 1);
        assert_eq!(StatusType::Offline as u8, 2);
        assert_eq!(StatusType::Error as u8, 3);
        assert_eq!(StatusType::PaperRoll as u8, 4);
    }

    #[test]
    fn transmit_status_encodes() {
        let cmd = TransmitStatus(StatusType::Printer);
        assert_eq!(cmd.encode(), vec![0x10, 0x04, 1]);
    }

    #[test]
    fn printer_status_parses() {
        let status = PrinterStatus::parse(0x00);
        assert!(!status.drawer_open);
        assert!(status.online);

        let status = PrinterStatus::parse(0x0C);
        assert!(status.drawer_open);
        assert!(!status.online);
    }

    #[test]
    fn asb_flags_to_byte() {
        assert_eq!(AsbFlags::none().to_byte(), 0);
        assert_eq!(AsbFlags::all().to_byte(), 0x0F);

        let flags = AsbFlags {
            drawer: true,
            online_offline: false,
            error: true,
            paper_roll: false,
        };
        assert_eq!(flags.to_byte(), 0x05);
    }

    #[test]
    fn enable_asb_encodes() {
        let cmd = EnableAsb(AsbFlags::all());
        assert_eq!(cmd.encode(), vec![0x1D, b'a', 0x0F]);
    }

    #[test]
    fn transmit_status_parses_response() {
        let cmd = TransmitStatus(StatusType::Printer);
        let response = cmd.parse_response(&[0x00]).unwrap();
        assert!(matches!(response, StatusResponse::Printer(_)));
    }
}
