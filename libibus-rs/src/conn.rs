use zbus::connection::Builder;

use crate::address;
use crate::dbus::IBusProxy;
use crate::error::{Error, Result};

/// Resolve the IBus address, establish a D-Bus connection, and perform the
/// `hello` handshake with the ibus-daemon.
///
/// Returns the raw [`zbus::Connection`] and an [`IBusProxy`] ready for use.
///
/// # Errors
///
/// Returns [`Error::Connection`] if the address cannot be resolved, the D-Bus
/// connection fails, or the hello handshake is rejected.
pub async fn connect() -> Result<(zbus::Connection, IBusProxy<'static>)> {
    let address_str = address::connect_address()?;

    let connection = Builder::address(address_str.as_str())
        .map_err(|e| Error::Connection(format!("Invalid address: {}", e)))?
        .build()
        .await
        .map_err(|e| Error::Connection(format!("Failed to connect to IBus: {}", e)))?;

    let mut bus_proxy = IBusProxy::new(&connection)
        .await
        .map_err(|e| Error::Connection(format!("Failed to get IBus proxy: {}", e)))?;

    bus_proxy
        .hello()
        .await
        .map_err(|e| Error::Connection(format!("IBus hello failed: {}", e)))?;

    Ok((connection, bus_proxy))
}
