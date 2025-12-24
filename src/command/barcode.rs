//! Barcode commands.
//!
//! All barcode types supported by the printer with validation.

use super::{Command, GS};
use crate::error::BarcodeError;
use miette::SourceSpan;

/// Set barcode height in dots.
///
/// ESC/POS: `GS h n` (0x1D 0x68 n)
/// Default: 162 dots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetBarcodeHeight(pub u8);

impl Default for SetBarcodeHeight {
    fn default() -> Self {
        Self(162)
    }
}

impl Command for SetBarcodeHeight {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'h', self.0.max(1)]
    }
}

/// Barcode module width.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarcodeWidth {
    /// Thinnest module (0.282mm).
    Thin = 2,
    /// Default width (0.423mm).
    #[default]
    Normal = 3,
    /// Medium width (0.564mm).
    Medium = 4,
    /// Wide width (0.706mm).
    Wide = 5,
    /// Widest module (0.847mm).
    ExtraWide = 6,
}

/// Set barcode module width.
///
/// ESC/POS: `GS w n` (0x1D 0x77 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetBarcodeWidth(pub BarcodeWidth);

impl Command for SetBarcodeWidth {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'w', self.0 as u8]
    }
}

/// HRI (Human Readable Interpretation) character position.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HriPosition {
    /// HRI not printed.
    #[default]
    None = 0,
    /// HRI printed above barcode.
    Above = 1,
    /// HRI printed below barcode.
    Below = 2,
    /// HRI printed both above and below.
    Both = 3,
}

/// Set HRI character position.
///
/// ESC/POS: `GS H n` (0x1D 0x48 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetHriPosition(pub HriPosition);

impl Command for SetHriPosition {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'H', self.0 as u8]
    }
}

/// HRI character font.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HriFont {
    /// Font A (12×24).
    #[default]
    A = 0,
    /// Font B (9×17).
    B = 1,
}

/// Set HRI character font.
///
/// ESC/POS: `GS f n` (0x1D 0x66 n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetHriFont(pub HriFont);

impl Command for SetHriFont {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'f', self.0 as u8]
    }
}

/// Barcode symbology.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeSystem {
    /// UPC-A - 11-12 digits, North American retail.
    UpcA = 65,
    /// UPC-E - 11-12 digits, compressed UPC-A.
    UpcE = 66,
    /// JAN-13/EAN-13 - 12-13 digits, international retail.
    Jan13 = 67,
    /// JAN-8/EAN-8 - 7-8 digits, small packages.
    Jan8 = 68,
    /// CODE39 - alphanumeric, variable length.
    Code39 = 69,
    /// ITF (Interleaved 2 of 5) - digits only, even length required.
    Itf = 70,
    /// CODABAR - digits and symbols, medical/library use.
    Codabar = 71,
    /// CODE93 - full ASCII, high density.
    Code93 = 72,
    /// CODE128 - full ASCII, very high density.
    Code128 = 73,
}

/// Print a barcode.
///
/// ESC/POS: `GS k m n d1...dn` (0x1D 0x6B m n d1...dn)
///
/// **Note:** Barcode configuration commands (height, width, HRI) must be
/// sent BEFORE this command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintBarcode {
    /// Barcode symbology.
    pub system: BarcodeSystem,
    /// Barcode data.
    pub data: Vec<u8>,
}

impl PrintBarcode {
    /// Create a new barcode with validation.
    ///
    /// # Errors
    ///
    /// Returns [`BarcodeError`] if the data is invalid for the barcode system.
    pub fn new(system: BarcodeSystem, data: impl Into<Vec<u8>>) -> Result<Self, BarcodeError> {
        let data = data.into();
        Self::validate(system, &data)?;
        Ok(Self {
            system,
            data,
        })
    }

    fn validate(system: BarcodeSystem, data: &[u8]) -> Result<(), BarcodeError> {
        let (min_len, max_len, name): (usize, usize, &'static str) = match system {
            BarcodeSystem::UpcA => (11, 12, "UPC-A"),
            BarcodeSystem::UpcE => (11, 12, "UPC-E"),
            BarcodeSystem::Jan13 => (12, 13, "JAN-13"),
            BarcodeSystem::Jan8 => (7, 8, "JAN-8"),
            BarcodeSystem::Code39 => (1, 255, "CODE39"),
            BarcodeSystem::Itf => (2, 255, "ITF"),
            BarcodeSystem::Codabar => (1, 255, "CODABAR"),
            BarcodeSystem::Code93 => (1, 255, "CODE93"),
            BarcodeSystem::Code128 => (2, 255, "CODE128"),
        };

        if data.len() < min_len || data.len() > max_len {
            return Err(BarcodeError::InvalidLength {
                system: name,
                actual: data.len(),
                min: min_len,
                max: max_len,
            });
        }

        // ITF requires even number of digits
        if matches!(system, BarcodeSystem::Itf) && !data.len().is_multiple_of(2) {
            return Err(BarcodeError::ItfRequiresEvenLength(data.len()));
        }

        // Validate characters
        for (i, &byte) in data.iter().enumerate() {
            let valid = match system {
                BarcodeSystem::UpcA
                | BarcodeSystem::UpcE
                | BarcodeSystem::Jan13
                | BarcodeSystem::Jan8
                | BarcodeSystem::Itf => byte.is_ascii_digit(),

                BarcodeSystem::Code39 => {
                    byte.is_ascii_digit()
                        || byte.is_ascii_uppercase()
                        || matches!(byte, b' ' | b'$' | b'%' | b'*' | b'+' | b'-' | b'.' | b'/')
                }

                BarcodeSystem::Codabar => {
                    byte.is_ascii_digit()
                        || (b'A'..=b'D').contains(&byte)
                        || matches!(byte, b'$' | b'+' | b'-' | b'.' | b'/' | b':')
                }

                BarcodeSystem::Code93 | BarcodeSystem::Code128 => byte <= 127,
            };

            if !valid {
                return Err(BarcodeError::InvalidCharacter {
                    data: String::from_utf8_lossy(data).into_owned(),
                    span: SourceSpan::from((i, 1)),
                    system: name,
                });
            }
        }

        Ok(())
    }
}

impl Command for PrintBarcode {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![GS, b'k', self.system as u8, self.data.len() as u8];
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barcode_height_default() {
        assert_eq!(SetBarcodeHeight::default().0, 162);
    }

    #[test]
    fn barcode_height_encodes() {
        let cmd = SetBarcodeHeight(100);
        assert_eq!(cmd.encode(), vec![0x1D, b'h', 100]);
    }

    #[test]
    fn barcode_width_values() {
        assert_eq!(BarcodeWidth::Thin as u8, 2);
        assert_eq!(BarcodeWidth::Normal as u8, 3);
        assert_eq!(BarcodeWidth::ExtraWide as u8, 6);
    }

    #[test]
    fn hri_position_values() {
        assert_eq!(HriPosition::None as u8, 0);
        assert_eq!(HriPosition::Below as u8, 2);
    }

    #[test]
    fn barcode_system_values() {
        assert_eq!(BarcodeSystem::UpcA as u8, 65);
        assert_eq!(BarcodeSystem::Code128 as u8, 73);
    }

    #[test]
    fn print_barcode_upc_a_valid() {
        let result = PrintBarcode::new(BarcodeSystem::UpcA, b"12345678901".to_vec());
        assert!(result.is_ok());
    }

    #[test]
    fn print_barcode_upc_a_invalid_chars() {
        let result = PrintBarcode::new(BarcodeSystem::UpcA, b"1234567890A".to_vec());
        assert!(matches!(result, Err(BarcodeError::InvalidCharacter { .. })));
    }

    #[test]
    fn print_barcode_itf_odd_length_rejected() {
        let result = PrintBarcode::new(BarcodeSystem::Itf, b"123".to_vec());
        assert!(matches!(result, Err(BarcodeError::ItfRequiresEvenLength(_))));
    }

    #[test]
    fn print_barcode_encodes() {
        let cmd = PrintBarcode::new(BarcodeSystem::Code128, b"{A123".to_vec()).unwrap();
        let encoded = cmd.encode();
        assert_eq!(encoded[0..4], [0x1D, b'k', 73, 5]);
    }
}
