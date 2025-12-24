//! 2D symbol commands (QR Code, PDF417).

use super::{Command, GS};
use crate::error::{Pdf417Error, QrCodeError};

/// QR Code model.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QrModel {
    /// Model 1 - Original QR Code.
    Model1 = 49,
    /// Model 2 - Enhanced QR Code (recommended).
    #[default]
    Model2 = 50,
}

/// QR Code error correction level.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QrErrorCorrection {
    /// Level L - approximately 7% recovery capacity.
    #[default]
    L = 48,
    /// Level M - approximately 15% recovery capacity.
    M = 49,
    /// Level Q - approximately 25% recovery capacity.
    Q = 50,
    /// Level H - approximately 30% recovery capacity.
    H = 51,
}

/// QR Code module size (1-8 dots per module).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QrModuleSize {
    /// 1 dot per module (smallest).
    Size1 = 1,
    /// 2 dots per module.
    Size2 = 2,
    /// 3 dots per module (default).
    #[default]
    Size3 = 3,
    /// 4 dots per module.
    Size4 = 4,
    /// 5 dots per module.
    Size5 = 5,
    /// 6 dots per module.
    Size6 = 6,
    /// 7 dots per module.
    Size7 = 7,
    /// 8 dots per module (largest).
    Size8 = 8,
}

/// Print a QR Code.
///
/// This is a compound command that sends multiple ESC/POS commands:
/// 1. Set model
/// 2. Set module size
/// 3. Set error correction
/// 4. Store data
/// 5. Print symbol
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintQrCode {
    /// QR Code model.
    pub model: QrModel,
    /// Module size.
    pub module_size: QrModuleSize,
    /// Error correction level.
    pub error_correction: QrErrorCorrection,
    /// Data to encode.
    pub data: Vec<u8>,
}

impl PrintQrCode {
    /// Create a QR code with default settings.
    pub fn new(data: impl Into<Vec<u8>>) -> Result<Self, QrCodeError> {
        let data = data.into();
        if data.is_empty() {
            return Err(QrCodeError::EmptyData);
        }
        if data.len() > 7089 {
            return Err(QrCodeError::DataTooLong(data.len()));
        }
        Ok(Self {
            model: QrModel::default(),
            module_size: QrModuleSize::default(),
            error_correction: QrErrorCorrection::default(),
            data,
        })
    }

    /// Set the QR code model.
    pub fn with_model(mut self, model: QrModel) -> Self {
        self.model = model;
        self
    }

    /// Set the module size.
    pub fn with_module_size(mut self, size: QrModuleSize) -> Self {
        self.module_size = size;
        self
    }

    /// Set the error correction level.
    pub fn with_error_correction(mut self, level: QrErrorCorrection) -> Self {
        self.error_correction = level;
        self
    }
}

impl Command for PrintQrCode {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64 + self.data.len());

        // Function 165: Select model
        bytes.extend_from_slice(&[GS, b'(', b'k', 4, 0, 49, 65, self.model as u8, 0]);

        // Function 167: Set module size
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 67, self.module_size as u8]);

        // Function 169: Set error correction
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 69, self.error_correction as u8]);

        // Function 180: Store data
        let data_len = self.data.len() + 3;
        let pl = (data_len & 0xFF) as u8;
        let ph = ((data_len >> 8) & 0xFF) as u8;
        bytes.extend_from_slice(&[GS, b'(', b'k', pl, ph, 49, 80, 48]);
        bytes.extend_from_slice(&self.data);

        // Function 181: Print symbol
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 81, 48]);

        bytes
    }
}

/// PDF417 module size (2-8 dots per module).
///
/// Used for both width and height of PDF417 modules.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pdf417ModuleSize {
    /// 2 dots per module (smallest).
    Size2 = 2,
    /// 3 dots per module (default).
    #[default]
    Size3 = 3,
    /// 4 dots per module.
    Size4 = 4,
    /// 5 dots per module.
    Size5 = 5,
    /// 6 dots per module.
    Size6 = 6,
    /// 7 dots per module.
    Size7 = 7,
    /// 8 dots per module (largest).
    Size8 = 8,
}

/// PDF417 column count configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pdf417Columns {
    /// Automatically determine column count (default).
    #[default]
    Auto,
    /// Manual column count (1-30).
    Manual(u8),
}

impl Pdf417Columns {
    /// Create a manual column count with validation.
    ///
    /// # Errors
    ///
    /// Returns [`Pdf417Error::InvalidColumnCount`] if `n` is not in range 1-30.
    pub fn manual(n: u8) -> Result<Self, Pdf417Error> {
        if !(1..=30).contains(&n) {
            return Err(Pdf417Error::InvalidColumnCount(n));
        }
        Ok(Pdf417Columns::Manual(n))
    }

    /// Encode to the byte value for the printer command.
    fn as_byte(self) -> u8 {
        match self {
            Pdf417Columns::Auto => 0,
            Pdf417Columns::Manual(n) => n,
        }
    }
}

/// PDF417 row count configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pdf417Rows {
    /// Automatically determine row count (default).
    #[default]
    Auto,
    /// Manual row count (3-90).
    Manual(u8),
}

impl Pdf417Rows {
    /// Create a manual row count with validation.
    ///
    /// # Errors
    ///
    /// Returns [`Pdf417Error::InvalidRowCount`] if `n` is not in range 3-90.
    pub fn manual(n: u8) -> Result<Self, Pdf417Error> {
        if !(3..=90).contains(&n) {
            return Err(Pdf417Error::InvalidRowCount(n));
        }
        Ok(Pdf417Rows::Manual(n))
    }

    /// Encode to the byte value for the printer command.
    fn as_byte(self) -> u8 {
        match self {
            Pdf417Rows::Auto => 0,
            Pdf417Rows::Manual(n) => n,
        }
    }
}

/// PDF417 error correction level (0-8).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pdf417ErrorCorrection {
    /// Level 0 - minimal error correction.
    Level0 = 48,
    /// Level 1.
    #[default]
    Level1 = 49,
    /// Level 2.
    Level2 = 50,
    /// Level 3.
    Level3 = 51,
    /// Level 4.
    Level4 = 52,
    /// Level 5.
    Level5 = 53,
    /// Level 6.
    Level6 = 54,
    /// Level 7.
    Level7 = 55,
    /// Level 8 - maximum error correction.
    Level8 = 56,
}

/// Print a PDF417 barcode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintPdf417 {
    /// Number of columns.
    pub columns: Pdf417Columns,
    /// Number of rows.
    pub rows: Pdf417Rows,
    /// Module width.
    pub module_width: Pdf417ModuleSize,
    /// Module height.
    pub module_height: Pdf417ModuleSize,
    /// Error correction level.
    pub error_correction: Pdf417ErrorCorrection,
    /// Data to encode.
    pub data: Vec<u8>,
}

impl PrintPdf417 {
    /// Create a PDF417 barcode with default settings.
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            columns: Pdf417Columns::default(),
            rows: Pdf417Rows::default(),
            module_width: Pdf417ModuleSize::default(),
            module_height: Pdf417ModuleSize::default(),
            error_correction: Pdf417ErrorCorrection::default(),
            data: data.into(),
        }
    }

    /// Set number of columns.
    pub fn with_columns(mut self, columns: Pdf417Columns) -> Self {
        self.columns = columns;
        self
    }

    /// Set number of rows.
    pub fn with_rows(mut self, rows: Pdf417Rows) -> Self {
        self.rows = rows;
        self
    }

    /// Set module width.
    pub fn with_module_width(mut self, width: Pdf417ModuleSize) -> Self {
        self.module_width = width;
        self
    }

    /// Set module height.
    pub fn with_module_height(mut self, height: Pdf417ModuleSize) -> Self {
        self.module_height = height;
        self
    }

    /// Set error correction level.
    pub fn with_error_correction(mut self, level: Pdf417ErrorCorrection) -> Self {
        self.error_correction = level;
        self
    }
}

impl Command for PrintPdf417 {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64 + self.data.len());

        // Function 65: Set columns
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 48, 65, self.columns.as_byte()]);

        // Function 66: Set rows
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 48, 66, self.rows.as_byte()]);

        // Function 67: Set module width
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 48, 67, self.module_width as u8]);

        // Function 68: Set module height
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 48, 68, self.module_height as u8]);

        // Function 69: Set error correction
        bytes.extend_from_slice(&[GS, b'(', b'k', 4, 0, 48, 69, 48, self.error_correction as u8]);

        // Function 80: Store data
        let data_len = self.data.len() + 3;
        let pl = (data_len & 0xFF) as u8;
        let ph = ((data_len >> 8) & 0xFF) as u8;
        bytes.extend_from_slice(&[GS, b'(', b'k', pl, ph, 48, 80, 48]);
        bytes.extend_from_slice(&self.data);

        // Function 81: Print symbol
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 48, 81, 48]);

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_model_values() {
        assert_eq!(QrModel::Model1 as u8, 49);
        assert_eq!(QrModel::Model2 as u8, 50);
    }

    #[test]
    fn qr_error_correction_values() {
        assert_eq!(QrErrorCorrection::L as u8, 48);
        assert_eq!(QrErrorCorrection::H as u8, 51);
    }

    #[test]
    fn qr_code_creates_with_defaults() {
        let qr = PrintQrCode::new(b"test".to_vec()).unwrap();
        assert_eq!(qr.model, QrModel::Model2);
        assert_eq!(qr.module_size, QrModuleSize::Size3);
        assert_eq!(qr.error_correction, QrErrorCorrection::L);
    }

    #[test]
    fn qr_module_size_values() {
        assert_eq!(QrModuleSize::Size1 as u8, 1);
        assert_eq!(QrModuleSize::Size8 as u8, 8);
    }

    #[test]
    fn qr_code_rejects_empty() {
        let result = PrintQrCode::new(vec![]);
        assert!(matches!(result, Err(QrCodeError::EmptyData)));
    }

    #[test]
    fn qr_code_encodes_commands() {
        let qr = PrintQrCode::new(b"Hello".to_vec()).unwrap();
        let encoded = qr.encode();

        // Should contain GS ( k sequences
        assert!(encoded.windows(3).any(|w| w == [GS, b'(', b'k']));

        // Should contain the data
        assert!(encoded.windows(5).any(|w| w == *b"Hello"));
    }

    #[test]
    fn pdf417_module_size_values() {
        assert_eq!(Pdf417ModuleSize::Size2 as u8, 2);
        assert_eq!(Pdf417ModuleSize::Size8 as u8, 8);
    }

    #[test]
    fn pdf417_columns_valid() {
        assert_eq!(Pdf417Columns::Auto.as_byte(), 0);
        assert_eq!(Pdf417Columns::manual(1).unwrap().as_byte(), 1);
        assert_eq!(Pdf417Columns::manual(15).unwrap().as_byte(), 15);
        assert_eq!(Pdf417Columns::manual(30).unwrap().as_byte(), 30);
    }

    #[test]
    fn pdf417_columns_invalid() {
        assert!(matches!(Pdf417Columns::manual(0), Err(Pdf417Error::InvalidColumnCount(0))));
        assert!(matches!(Pdf417Columns::manual(31), Err(Pdf417Error::InvalidColumnCount(31))));
    }

    #[test]
    fn pdf417_rows_valid() {
        assert_eq!(Pdf417Rows::Auto.as_byte(), 0);
        assert_eq!(Pdf417Rows::manual(3).unwrap().as_byte(), 3);
        assert_eq!(Pdf417Rows::manual(45).unwrap().as_byte(), 45);
        assert_eq!(Pdf417Rows::manual(90).unwrap().as_byte(), 90);
    }

    #[test]
    fn pdf417_rows_invalid() {
        assert!(matches!(Pdf417Rows::manual(2), Err(Pdf417Error::InvalidRowCount(2))));
        assert!(matches!(Pdf417Rows::manual(91), Err(Pdf417Error::InvalidRowCount(91))));
    }

    #[test]
    fn pdf417_creates_with_defaults() {
        let pdf = PrintPdf417::new(b"test".to_vec());
        assert_eq!(pdf.columns, Pdf417Columns::Auto);
        assert_eq!(pdf.rows, Pdf417Rows::Auto);
        assert_eq!(pdf.module_width, Pdf417ModuleSize::Size3);
        assert_eq!(pdf.module_height, Pdf417ModuleSize::Size3);
    }

    #[test]
    fn pdf417_encodes_commands() {
        let pdf = PrintPdf417::new(b"Hello".to_vec());
        let encoded = pdf.encode();

        // Should contain GS ( k sequences
        assert!(encoded.windows(3).any(|w| w == [GS, b'(', b'k']));

        // Should contain the data
        assert!(encoded.windows(5).any(|w| w == *b"Hello"));
    }
}
