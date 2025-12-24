# Task 3: Printer Control Commands

> **Context:** See [index.md](./index.md) for architecture and rationale

**Files:**
- Create: `src/command/printer_control.rs`
- Modify: `src/command/mod.rs` (add module)

---

## Step 1: Add module declaration

Update `src/command/mod.rs`:

```rust
pub mod printer_control;
```

---

## Step 2: Write failing tests

Create `src/command/printer_control.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn initialize_encodes_correctly() {
        let cmd = Initialize;
        assert_eq!(cmd.encode(), vec![0x1B, b'@']);
    }

    #[test]
    fn select_peripheral_printer_only() {
        let cmd = SelectPeripheral(PeripheralDevice::PrinterOnly);
        assert_eq!(cmd.encode(), vec![0x1B, b'=', 1]);
    }

    #[test]
    fn generate_pulse_pin2() {
        let cmd = GeneratePulse {
            pin: DrawerPin::Pin2,
            on_time_ms: 200,
            off_time_ms: 200,
        };
        let encoded = cmd.encode();
        assert_eq!(encoded[0..3], [0x1B, b'p', 0]);
    }
}
```

---

## Step 3: Run tests to verify they fail

```bash
cargo test -p bixolon command::printer_control
```

Expected: Compilation error - types not defined

---

## Step 4: Implement printer control commands

Update `src/command/printer_control.rs`:

```rust
//! Printer control commands.
//!
//! Commands for initializing the printer, selecting peripherals, and generating pulses.

use super::{Command, ESC};

/// Initialize the printer.
///
/// Clears the print buffer and resets the printer to its power-on state.
/// All settings return to their default values.
///
/// ESC/POS: `ESC @` (0x1B 0x40)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Initialize;

impl Command for Initialize {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'@']
    }
}

/// Peripheral device selection.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PeripheralDevice {
    /// Send data to printer only.
    #[default]
    PrinterOnly = 1,
    /// Send data to customer display only.
    DisplayOnly = 2,
    /// Send data to both printer and customer display.
    Both = 3,
}

/// Select peripheral device for data transmission.
///
/// Selects which device(s) receive subsequent data.
///
/// ESC/POS: `ESC = n` (0x1B 0x3D n)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectPeripheral(pub PeripheralDevice);

impl Default for SelectPeripheral {
    fn default() -> Self {
        Self(PeripheralDevice::PrinterOnly)
    }
}

impl Command for SelectPeripheral {
    fn encode(&self) -> Vec<u8> {
        vec![ESC, b'=', self.0 as u8]
    }
}

/// Drawer kick-out connector pin selection.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerPin {
    /// Drawer kick-out connector pin 2.
    #[default]
    Pin2 = 0,
    /// Drawer kick-out connector pin 5.
    Pin5 = 1,
}

/// Generate pulse on drawer kick-out connector.
///
/// Outputs a pulse to the specified pin on the drawer connector.
/// Used to open cash drawers.
///
/// ESC/POS: `ESC p m t1 t2` (0x1B 0x70 m t1 t2)
///
/// # Parameters
///
/// - `pin`: Which connector pin to pulse
/// - `on_time_ms`: Pulse ON time in milliseconds (will be rounded to nearest 2ms)
/// - `off_time_ms`: Pulse OFF time in milliseconds (will be rounded to nearest 2ms)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratePulse {
    /// Which pin to generate pulse on.
    pub pin: DrawerPin,
    /// ON time in milliseconds (actual = value × 2ms, max 510ms).
    pub on_time_ms: u16,
    /// OFF time in milliseconds (actual = value × 2ms, max 510ms).
    pub off_time_ms: u16,
}

impl GeneratePulse {
    /// Create a new pulse command with the given parameters.
    ///
    /// Times are clamped to 0-510ms and rounded to nearest 2ms.
    pub fn new(pin: DrawerPin, on_time_ms: u16, off_time_ms: u16) -> Self {
        Self {
            pin,
            on_time_ms: on_time_ms.min(510),
            off_time_ms: off_time_ms.min(510),
        }
    }

    /// Open a standard cash drawer with default timing.
    ///
    /// Uses 200ms on time and 200ms off time on pin 2.
    pub fn open_drawer() -> Self {
        Self {
            pin: DrawerPin::Pin2,
            on_time_ms: 200,
            off_time_ms: 200,
        }
    }
}

impl Command for GeneratePulse {
    fn encode(&self) -> Vec<u8> {
        // Time values are in units of 2ms
        let t1 = (self.on_time_ms / 2).min(255) as u8;
        let t2 = (self.off_time_ms / 2).min(255) as u8;
        vec![ESC, b'p', self.pin as u8, t1, t2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_encodes_correctly() {
        let cmd = Initialize;
        assert_eq!(cmd.encode(), vec![0x1B, b'@']);
    }

    #[test]
    fn select_peripheral_printer_only() {
        let cmd = SelectPeripheral(PeripheralDevice::PrinterOnly);
        assert_eq!(cmd.encode(), vec![0x1B, b'=', 1]);
    }

    #[test]
    fn select_peripheral_display_only() {
        let cmd = SelectPeripheral(PeripheralDevice::DisplayOnly);
        assert_eq!(cmd.encode(), vec![0x1B, b'=', 2]);
    }

    #[test]
    fn select_peripheral_both() {
        let cmd = SelectPeripheral(PeripheralDevice::Both);
        assert_eq!(cmd.encode(), vec![0x1B, b'=', 3]);
    }

    #[test]
    fn generate_pulse_pin2() {
        let cmd = GeneratePulse {
            pin: DrawerPin::Pin2,
            on_time_ms: 200,
            off_time_ms: 200,
        };
        let encoded = cmd.encode();
        assert_eq!(encoded, vec![0x1B, b'p', 0, 100, 100]);
    }

    #[test]
    fn generate_pulse_pin5() {
        let cmd = GeneratePulse {
            pin: DrawerPin::Pin5,
            on_time_ms: 100,
            off_time_ms: 50,
        };
        let encoded = cmd.encode();
        assert_eq!(encoded, vec![0x1B, b'p', 1, 50, 25]);
    }

    #[test]
    fn generate_pulse_clamps_time() {
        let cmd = GeneratePulse::new(DrawerPin::Pin2, 1000, 1000);
        let encoded = cmd.encode();
        // 510ms / 2 = 255
        assert_eq!(encoded, vec![0x1B, b'p', 0, 255, 255]);
    }

    #[test]
    fn open_drawer_convenience() {
        let cmd = GeneratePulse::open_drawer();
        assert_eq!(cmd.pin, DrawerPin::Pin2);
        assert_eq!(cmd.on_time_ms, 200);
        assert_eq!(cmd.off_time_ms, 200);
    }

    #[test]
    fn peripheral_device_default_is_printer() {
        assert_eq!(PeripheralDevice::default(), PeripheralDevice::PrinterOnly);
    }

    #[test]
    fn drawer_pin_default_is_pin2() {
        assert_eq!(DrawerPin::default(), DrawerPin::Pin2);
    }
}
```

---

## Step 5: Run tests to verify they pass

```bash
cargo test -p bixolon command::printer_control
```

Expected: All tests pass

---

## Step 6: Commit

```bash
git add src/command/printer_control.rs src/command/mod.rs
git commit -m "feat: add printer control commands (Initialize, SelectPeripheral, GeneratePulse)"
```
