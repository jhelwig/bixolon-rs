//! Image commands.

use super::{Command, ESC, GS};

/// Bit image mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitImageMode {
    /// 8-dot single density (60 dpi V, 90 dpi H).
    SingleDensity8 = 0,
    /// 8-dot double density (60 dpi V, 180 dpi H).
    DoubleDensity8 = 1,
    /// 24-dot single density (180 dpi V, 90 dpi H).
    SingleDensity24 = 32,
    /// 24-dot double density (180 dpi V, 180 dpi H).
    #[default]
    DoubleDensity24 = 33,
}

/// Select bit image mode and print.
///
/// ESC/POS: `ESC * m nL nH d1...dk`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectBitImageMode {
    /// Image mode.
    pub mode: BitImageMode,
    /// Number of data bytes per column.
    pub width: u16,
    /// Image data.
    pub data: Vec<u8>,
}

impl Command for SelectBitImageMode {
    fn encode(&self) -> Vec<u8> {
        let nl = (self.width & 0xFF) as u8;
        let nh = ((self.width >> 8) & 0xFF) as u8;
        let mut bytes = vec![ESC, b'*', self.mode as u8, nl, nh];
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// Raster image mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RasterImageMode {
    /// Normal (180 dpi).
    #[default]
    Normal = 0,
    /// Double width (90 dpi H).
    DoubleWidth = 1,
    /// Double height (90 dpi V).
    DoubleHeight = 2,
    /// Quadruple (90 dpi both).
    Quadruple = 3,
}

/// Print raster bit image.
///
/// ESC/POS: `GS v 0 m xL xH yL yH d1...dk`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintRasterImage {
    /// Image mode/density.
    pub mode: RasterImageMode,
    /// Width in bytes (8 pixels per byte).
    pub width_bytes: u16,
    /// Height in dots.
    pub height_dots: u16,
    /// Image data (1 bit per pixel, MSB first).
    pub data: Vec<u8>,
}

impl PrintRasterImage {
    /// Create a new raster image.
    pub fn new(width_bytes: u16, height_dots: u16, data: Vec<u8>) -> Self {
        Self {
            mode: RasterImageMode::Normal,
            width_bytes,
            height_dots,
            data,
        }
    }

    /// Set the raster image mode.
    pub fn with_mode(mut self, mode: RasterImageMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Command for PrintRasterImage {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![
            GS,
            b'v',
            b'0',
            self.mode as u8,
            (self.width_bytes & 0xFF) as u8,
            ((self.width_bytes >> 8) & 0xFF) as u8,
            (self.height_dots & 0xFF) as u8,
            ((self.height_dots >> 8) & 0xFF) as u8,
        ];
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// Define downloaded bit image.
///
/// ESC/POS: `GS * x y d1...dk`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefineDownloadedImage {
    /// Width in bytes (1-255).
    pub width_bytes: u8,
    /// Height in bytes (1-48).
    pub height_bytes: u8,
    /// Image data.
    pub data: Vec<u8>,
}

impl Command for DefineDownloadedImage {
    fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![GS, b'*', self.width_bytes, self.height_bytes];
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// Print downloaded image mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadedImageMode {
    /// Normal.
    #[default]
    Normal = 0,
    /// Double width.
    DoubleWidth = 1,
    /// Double height.
    DoubleHeight = 2,
    /// Double width and height.
    Quadruple = 3,
}

/// Print the downloaded bit image.
///
/// ESC/POS: `GS / m`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrintDownloadedImage(pub DownloadedImageMode);

impl Command for PrintDownloadedImage {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'/', self.0 as u8]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_image_mode_values() {
        assert_eq!(BitImageMode::SingleDensity8 as u8, 0);
        assert_eq!(BitImageMode::DoubleDensity24 as u8, 33);
    }

    #[test]
    fn select_bit_image_mode_encodes() {
        let cmd = SelectBitImageMode {
            mode: BitImageMode::DoubleDensity24,
            width: 100,
            data: vec![0xFF; 100],
        };
        let encoded = cmd.encode();
        assert_eq!(&encoded[0..5], &[0x1B, b'*', 33, 100, 0]);
    }

    #[test]
    fn raster_image_mode_values() {
        assert_eq!(RasterImageMode::Normal as u8, 0);
        assert_eq!(RasterImageMode::Quadruple as u8, 3);
    }

    #[test]
    fn print_raster_image_encodes() {
        let cmd = PrintRasterImage {
            mode: RasterImageMode::Normal,
            width_bytes: 64,
            height_dots: 100,
            data: vec![0xFF; 6400],
        };
        let encoded = cmd.encode();
        assert_eq!(&encoded[0..8], &[0x1D, b'v', b'0', 0, 64, 0, 100, 0]);
    }

    #[test]
    fn define_downloaded_image_encodes() {
        let cmd = DefineDownloadedImage {
            width_bytes: 10,
            height_bytes: 20,
            data: vec![0xFF; 200],
        };
        let encoded = cmd.encode();
        assert_eq!(&encoded[0..4], &[0x1D, b'*', 10, 20]);
    }

    #[test]
    fn print_downloaded_image_encodes() {
        let cmd = PrintDownloadedImage(DownloadedImageMode::DoubleWidth);
        assert_eq!(cmd.encode(), vec![0x1D, b'/', 1]);
    }
}
