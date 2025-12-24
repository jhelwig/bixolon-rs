//! Printer interface.
//!
//! Provides [`Printer`] for synchronous printing.

mod sync;

pub use sync::Printer;

#[cfg(feature = "async")]
mod r#async;

#[cfg(feature = "async")]
pub use r#async::AsyncPrinter;
