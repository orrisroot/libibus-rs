use zbus::Connection;
use zbus::connection::Builder;

use crate::address;
use crate::dbus::IBusProxy;
use crate::error::{Error, Result};

/// Resolve the IBus address, establish a D-Bus connection to the ibus-daemon's
/// private bus, and create an [`IBusProxy`] for `org.freedesktop.IBus`.
///
/// The ibus-daemon serves its API (`RegisterComponent`, `SetGlobalEngine`,
/// etc.) on this private bus. Engines use this bus to register components and
/// communicate with the daemon.
///
/// # Errors
///
/// Returns [`Error::Connection`] if the address cannot be resolved, the D-Bus
/// connection fails, or the proxy cannot be created.
pub async fn connect() -> Result<(Connection, IBusProxy<'static>)> {
    let address_str = address::connect_address()?;

    let connection = Builder::address(address_str.as_str())
        .map_err(|e| Error::Connection(format!("Invalid address: {}", e)))?
        .build()
        .await
        .map_err(|e| Error::Connection(format!("Failed to connect to IBus: {}", e)))?;

    let bus_proxy = IBusProxy::new(&connection)
        .await
        .map_err(|e| Error::Connection(format!("Failed to get IBus proxy: {}", e)))?;

    Ok((connection, bus_proxy))
}

/// Connect to the D-Bus session bus (no proxy created).
///
/// Use this connection to register your engine factory so the ibus-daemon can
/// discover it via `NameOwnerChanged` on the session bus.
///
/// # Errors
///
/// Returns [`Error::Connection`] if the session bus cannot be reached.
pub async fn connect_session() -> Result<Connection> {
    Connection::session()
        .await
        .map_err(|e| Error::Connection(format!("Failed to connect to session bus: {}", e)))
}
