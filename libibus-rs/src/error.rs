use thiserror::Error;

/// Errors returned by the libibus-rs library.
///
/// Wraps D-Bus, connection, component, and engine-level failures.
#[derive(Error, Debug)]
pub enum Error {
    /// Wraps an underlying [`zbus::Error`].
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    /// Component registration or manipulation failed.
    #[error("Component error: {0}")]
    Component(String),

    /// Engine creation or invocation failed.
    #[error("Engine error: {0}")]
    Engine(String),

    /// D-Bus connection or IBus handshake failed.
    #[error("Connection error: {0}")]
    Connection(String),

    /// IBus address resolution failed.
    #[error("Address error: {0}")]
    Address(String),

    /// Operation attempted without an active IBus connection.
    #[error("Not connected to IBus daemon")]
    NotConnected,

    /// Requested engine name was not found.
    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    /// An argument passed to an API function was invalid.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Wraps an [`std::io::Error`].
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias for [`Result<T, libibus_rs::Error>`](enum@Error).
pub type Result<T> = std::result::Result<T, Error>;
