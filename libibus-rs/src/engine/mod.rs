//! IBus engine D-Bus interface.
//!
//! The engine module is split into several sub-modules:
//!
//! * [`trait_`] — the [`EngineImpl`] trait (user-implementable, no zbus dependency).
//! * [`handle`] — [`EngineHandle`] for emitting D-Bus signals.
//! * [`dbus`] — `#[interface]` implementation and signal declarations.
//! * `descriptor` — internal `Engine` struct (pub(crate)).
//!
//! See [`crate::factory`] for how to register engines with the daemon.

pub mod dbus;
mod descriptor;
pub mod handle;
pub mod trait_;

pub use handle::EngineHandle;
pub use trait_::EngineImpl;

pub(crate) use descriptor::Engine;
