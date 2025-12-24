//! Macro definition and execution commands.
//!
//! Macros allow storing a sequence of commands and replaying them.
//! Maximum macro size is 2048 bytes.

use super::{Command, GS};

/// Toggle macro definition mode.
///
/// ESC/POS: `GS :` (0x1D 0x3A)
///
/// First call starts recording, second call ends recording.
/// All commands between are stored in the macro buffer.
///
/// # Example
///
/// ```ignore
/// printer.send(&ToggleMacroDefinition)?;  // Start recording
/// printer.send(&SetEmphasized(true))?;
/// printer.send(&PrintText::new("HEADER"))?;
/// printer.send(&SetEmphasized(false))?;
/// printer.send(&ToggleMacroDefinition)?;  // End recording
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToggleMacroDefinition;

impl Command for ToggleMacroDefinition {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b':']
    }
}

/// Macro execution mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacroExecutionMode {
    /// Execute macro continuously.
    #[default]
    Continuous = 0,
    /// Execute macro, wait for paper feed button.
    WaitForButton = 1,
}

/// Execute the defined macro.
///
/// ESC/POS: `GS ^ r t m` (0x1D 0x5E r t m)
///
/// # Parameters
///
/// - `times`: Number of times to execute (1-255)
/// - `wait_ms`: Wait time between executions in 100ms units (0-255)
/// - `mode`: Execution mode (continuous or wait for button)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecuteMacro {
    /// Number of times to execute (1-255).
    pub times: u8,
    /// Wait time between executions in 100ms units.
    pub wait_100ms: u8,
    /// Execution mode.
    pub mode: MacroExecutionMode,
}

impl ExecuteMacro {
    /// Execute macro once with no wait.
    pub const fn once() -> Self {
        Self {
            times: 1,
            wait_100ms: 0,
            mode: MacroExecutionMode::Continuous,
        }
    }

    /// Execute macro multiple times with optional wait.
    pub const fn repeat(times: u8, wait_100ms: u8) -> Self {
        Self {
            times,
            wait_100ms,
            mode: MacroExecutionMode::Continuous,
        }
    }

    /// Set execution mode.
    pub const fn with_mode(mut self, mode: MacroExecutionMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for ExecuteMacro {
    fn default() -> Self {
        Self::once()
    }
}

impl Command for ExecuteMacro {
    fn encode(&self) -> Vec<u8> {
        vec![GS, b'^', self.times.max(1), self.wait_100ms, self.mode as u8]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_macro_definition_encodes() {
        let cmd = ToggleMacroDefinition;
        assert_eq!(cmd.encode(), vec![0x1D, b':']);
    }

    #[test]
    fn macro_execution_mode_values() {
        assert_eq!(MacroExecutionMode::Continuous as u8, 0);
        assert_eq!(MacroExecutionMode::WaitForButton as u8, 1);
    }

    #[test]
    fn execute_macro_once_encodes() {
        let cmd = ExecuteMacro::once();
        assert_eq!(cmd.encode(), vec![0x1D, b'^', 1, 0, 0]);
    }

    #[test]
    fn execute_macro_repeat_encodes() {
        let cmd = ExecuteMacro::repeat(5, 10);
        assert_eq!(cmd.encode(), vec![0x1D, b'^', 5, 10, 0]);
    }

    #[test]
    fn execute_macro_with_button_wait() {
        let cmd = ExecuteMacro::once().with_mode(MacroExecutionMode::WaitForButton);
        assert_eq!(cmd.encode(), vec![0x1D, b'^', 1, 0, 1]);
    }

    #[test]
    fn execute_macro_enforces_min_times() {
        let cmd = ExecuteMacro {
            times: 0,
            wait_100ms: 0,
            mode: MacroExecutionMode::Continuous,
        };
        let encoded = cmd.encode();
        assert_eq!(encoded[2], 1); // Should be clamped to 1
    }
}
