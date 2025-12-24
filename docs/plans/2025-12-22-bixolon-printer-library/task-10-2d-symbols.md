# Task 10: 2D Symbol Commands (QR Code, PDF417)

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/symbol.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Commands to Implement

### QR Code (cn=49)
- Set model (Function 165)
- Set module size (Function 167)
- Set error correction level (Function 169)
- Store data (Function 180)
- Print symbol (Function 181)

### PDF417 (cn=48)
- Set columns/rows
- Set module width/height
- Set error correction level
- Store data and print

---

## Implementation

```rust
//! 2D symbol commands (QR Code, PDF417).

use super::{Command, GS};

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
    /// Module size (1-8).
    pub module_size: u8,
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
            module_size: 3,
            error_correction: QrErrorCorrection::default(),
            data,
        })
    }

    /// Set the QR code model.
    pub fn with_model(mut self, model: QrModel) -> Self {
        self.model = model;
        self
    }

    /// Set the module size (1-8).
    pub fn with_module_size(mut self, size: u8) -> Result<Self, QrCodeError> {
        if !(1..=8).contains(&size) {
            return Err(QrCodeError::InvalidModuleSize(size));
        }
        self.module_size = size;
        Ok(self)
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
        bytes.extend_from_slice(&[GS, b'(', b'k', 3, 0, 49, 67, self.module_size]);

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

/// QR Code errors.
#[derive(Debug, thiserror::Error)]
pub enum QrCodeError {
    /// Data is empty.
    #[error("QR code data cannot be empty")]
    EmptyData,
    /// Data exceeds maximum length.
    #[error("QR code data too long: {0} bytes (max 7089)")]
    DataTooLong(usize),
    /// Invalid module size.
    #[error("invalid module size {0} (must be 1-8)")]
    InvalidModuleSize(u8),
}

// PDF417 implementation follows similar pattern...

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
        assert_eq!(qr.module_size, 3);
        assert_eq!(qr.error_correction, QrErrorCorrection::L);
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
}
```

---

## Commit

```bash
git add src/command/symbol.rs src/command/mod.rs
git commit -m "feat: add QR code and PDF417 commands"
```
