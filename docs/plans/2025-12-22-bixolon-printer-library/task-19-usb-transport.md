# Task 19: USB Transport

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/transport/mod.rs`
- Create: `src/transport/usb.rs`
- Modify: `src/lib.rs` (add module)
- Modify: `Cargo.toml` (add rusb feature)

**Important:** Feature-gated behind `rusb` feature. Provides convenience wrappers around rusb for USB communication.

---

## Cargo.toml Feature

```toml
[features]
default = []
async = ["dep:tokio"]
rusb = ["dep:rusb"]

[dependencies]
rusb = { version = "0.9", optional = true }
```

---

## Implementation

```rust
// src/transport/mod.rs
//! Transport layer for printer communication.

#[cfg(feature = "rusb")]
pub mod usb;
```

```rust
// src/transport/usb.rs
//! USB transport using rusb.
//!
//! Provides convenience types for USB communication with Bixolon printers.
//!
//! # Example
//!
//! ```ignore
//! use bixolon::transport::usb::UsbPrinter;
//! use bixolon::printer::Printer;
//!
//! // Find and open the printer
//! let usb = UsbPrinter::find_bixolon()?.open()?;
//! let (reader, writer) = usb.split();
//! let mut printer = Printer::with_reader(writer, reader);
//! ```

use std::io::{Read, Write};
use std::time::Duration;

use rusb::{Context, Device, DeviceHandle, Direction, TransferType, UsbContext};

use crate::error::UsbError;

/// Bixolon USB Vendor ID.
pub const BIXOLON_VENDOR_ID: u16 = 0x1504;

/// SRP-350plus USB Product ID.
pub const SRP350PLUS_PRODUCT_ID: u16 = 0x0027;

/// Default USB timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A discovered USB printer device.
pub struct UsbPrinterDevice {
    device: Device<Context>,
    vendor_id: u16,
    product_id: u16,
}

impl UsbPrinterDevice {
    /// Get the vendor ID.
    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    /// Get the product ID.
    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    /// Open the device for communication.
    pub fn open(self) -> Result<UsbPrinter, UsbError> {
        let handle = self.device.open()?;

        // Find the bulk endpoints
        let config = self.device.active_config_descriptor()?;

        let mut write_endpoint = None;
        let mut read_endpoint = None;
        let mut interface_number = 0;

        'outer: for interface in config.interfaces() {
            for descriptor in interface.descriptors() {
                for endpoint in descriptor.endpoint_descriptors() {
                    if endpoint.transfer_type() == TransferType::Bulk {
                        match endpoint.direction() {
                            Direction::Out => {
                                write_endpoint = Some(endpoint.address());
                                interface_number = interface.number();
                            }
                            Direction::In => {
                                read_endpoint = Some(endpoint.address());
                            }
                        }
                    }
                }
                if write_endpoint.is_some() && read_endpoint.is_some() {
                    break 'outer;
                }
            }
        }

        let write_endpoint = write_endpoint.ok_or(UsbError::NoWriteEndpoint)?;
        let read_endpoint = read_endpoint.ok_or(UsbError::NoReadEndpoint)?;

        Ok(UsbPrinter {
            handle,
            write_endpoint,
            read_endpoint,
            interface_number,
            timeout: DEFAULT_TIMEOUT,
            claimed: false,
        })
    }
}

/// An opened USB printer.
pub struct UsbPrinter {
    handle: DeviceHandle<Context>,
    write_endpoint: u8,
    read_endpoint: u8,
    interface_number: u8,
    timeout: Duration,
    claimed: bool,
}

impl UsbPrinter {
    /// Find a Bixolon printer by vendor ID.
    pub fn find_bixolon() -> Result<UsbPrinterDevice, UsbError> {
        Self::find_by_vendor(BIXOLON_VENDOR_ID)
    }

    /// Find a printer by vendor ID.
    pub fn find_by_vendor(vendor_id: u16) -> Result<UsbPrinterDevice, UsbError> {
        let context = Context::new()?;

        for device in context.devices()?.iter() {
            let desc = device.device_descriptor()?;
            if desc.vendor_id() == vendor_id {
                return Ok(UsbPrinterDevice {
                    device,
                    vendor_id: desc.vendor_id(),
                    product_id: desc.product_id(),
                });
            }
        }

        Err(UsbError::DeviceNotFound)
    }

    /// Find a printer by vendor and product ID.
    pub fn find_by_ids(vendor_id: u16, product_id: u16) -> Result<UsbPrinterDevice, UsbError> {
        let context = Context::new()?;

        for device in context.devices()?.iter() {
            let desc = device.device_descriptor()?;
            if desc.vendor_id() == vendor_id && desc.product_id() == product_id {
                return Ok(UsbPrinterDevice {
                    device,
                    vendor_id: desc.vendor_id(),
                    product_id: desc.product_id(),
                });
            }
        }

        Err(UsbError::DeviceNotFound)
    }

    /// List all USB printers from known vendors.
    pub fn list_printers() -> Result<Vec<UsbPrinterDevice>, UsbError> {
        let context = Context::new()?;
        let mut printers = Vec::new();

        for device in context.devices()?.iter() {
            let desc = device.device_descriptor()?;
            // Check for known printer vendor IDs
            if desc.vendor_id() == BIXOLON_VENDOR_ID {
                printers.push(UsbPrinterDevice {
                    device,
                    vendor_id: desc.vendor_id(),
                    product_id: desc.product_id(),
                });
            }
        }

        Ok(printers)
    }

    /// Set the USB transfer timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Claim the USB interface.
    ///
    /// Must be called before reading or writing.
    pub fn claim(&mut self) -> Result<(), UsbError> {
        if self.claimed {
            return Ok(());
        }

        // Detach kernel driver if attached (Linux)
        #[cfg(target_os = "linux")]
        if self.handle.kernel_driver_active(self.interface_number)? {
            self.handle.detach_kernel_driver(self.interface_number)?;
        }

        self.handle.claim_interface(self.interface_number)?;
        self.claimed = true;
        Ok(())
    }

    /// Release the USB interface.
    pub fn release(&mut self) -> Result<(), UsbError> {
        if !self.claimed {
            return Ok(());
        }

        self.handle.release_interface(self.interface_number)?;
        self.claimed = false;
        Ok(())
    }

    /// Split into separate reader and writer handles.
    ///
    /// Claims the interface if not already claimed.
    pub fn split(mut self) -> Result<(UsbReader, UsbWriter), UsbError> {
        self.claim()?;

        // We need to use Arc for shared handle
        use std::sync::Arc;
        let handle = Arc::new(self.handle);

        Ok((
            UsbReader {
                handle: Arc::clone(&handle),
                endpoint: self.read_endpoint,
                timeout: self.timeout,
            },
            UsbWriter {
                handle,
                endpoint: self.write_endpoint,
                timeout: self.timeout,
            },
        ))
    }
}

impl Drop for UsbPrinter {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// USB reader handle.
pub struct UsbReader {
    handle: std::sync::Arc<DeviceHandle<Context>>,
    endpoint: u8,
    timeout: Duration,
}

impl Read for UsbReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.handle
            .read_bulk(self.endpoint, buf, self.timeout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// USB writer handle.
pub struct UsbWriter {
    handle: std::sync::Arc<DeviceHandle<Context>>,
    endpoint: u8,
    timeout: Duration,
}

impl Write for UsbWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.handle
            .write_bulk(self.endpoint, buf, self.timeout)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // USB bulk transfers are complete when write returns
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_defined() {
        assert_eq!(BIXOLON_VENDOR_ID, 0x1504);
        assert_eq!(SRP350PLUS_PRODUCT_ID, 0x0027);
    }

    // Hardware tests would go in the hardware-tests feature
}
```

---

## Add to error.rs

```rust
/// USB transport errors.
#[derive(Debug, thiserror::Error)]
pub enum UsbError {
    /// rusb error.
    #[error("USB error: {0}")]
    Rusb(#[from] rusb::Error),

    /// Device not found.
    #[error("USB printer device not found")]
    DeviceNotFound,

    /// No write endpoint found.
    #[error("no bulk OUT endpoint found")]
    NoWriteEndpoint,

    /// No read endpoint found.
    #[error("no bulk IN endpoint found")]
    NoReadEndpoint,
}
```

---

## Commit

```bash
git add src/transport/mod.rs src/transport/usb.rs src/error.rs src/lib.rs Cargo.toml
git commit -m "feat: add USB transport with rusb (feature-gated)"
```
