use crate::component::Component;
use crate::conn;
use crate::dbus::IBusProxy;
use crate::error::{Error, Result};
use crate::input_context::InputContext;
use crate::serializable::IBusSerializable;

/// A client connection to the ibus-daemon.
///
/// `Bus` manages the D-Bus connection lifecycle and provides high-level methods
/// for registering components, switching engines, and querying the daemon.
///
/// # Example
///
/// ```rust,no_run
/// use libibus_rs::Bus;
///
/// # async fn example() -> libibus_rs::Result<()> {
/// let mut bus = Bus::new();
/// bus.connect().await?;
/// println!("address: {}", bus.get_address().await?);
/// # Ok(())
/// # }
/// ```
pub struct Bus {
    connection: Option<zbus::Connection>,
    bus_proxy: Option<IBusProxy<'static>>,
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    /// Create a disconnected `Bus`.
    pub fn new() -> Self {
        Self {
            connection: None,
            bus_proxy: None,
        }
    }

    /// Resolve the IBus address, establish a D-Bus connection, and perform the
    /// `hello` handshake.
    pub async fn connect(&mut self) -> Result<()> {
        let (connection, bus_proxy) = conn::connect().await?;
        self.connection = Some(connection);
        self.bus_proxy = Some(bus_proxy);
        Ok(())
    }

    /// Whether the bus is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Access the underlying [`zbus::Connection`].
    ///
    /// Returns [`Error::NotConnected`] if [`connect`](Self::connect) has not been called.
    pub fn connection(&self) -> Result<&zbus::Connection> {
        self.connection.as_ref().ok_or(Error::NotConnected)
    }

    /// Access a clone of the underlying D-Bus proxy.
    ///
    /// Returns [`Error::NotConnected`] if [`connect`](Self::connect) has not been called.
    pub fn bus_proxy(&self) -> Result<IBusProxy<'static>> {
        self.bus_proxy.clone().ok_or(Error::NotConnected)
    }

    /// Register an input method component (and its engines) with the daemon.
    pub async fn register_component(&self, component: &Component) -> Result<()> {
        let proxy = self.bus_proxy()?;
        proxy
            .register_component(&component.to_value())
            .await
            .map_err(|e| Error::Component(format!("Failed to register component: {}", e)))?;
        Ok(())
    }

    /// Return the address string of the ibus-daemon.
    pub async fn get_address(&self) -> Result<String> {
        let proxy = self.bus_proxy()?;
        proxy
            .get_address()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get address: {}", e)))
    }

    /// Switch the global input method engine.
    pub async fn set_global_engine(&self, engine_name: &str) -> Result<()> {
        let proxy = self.bus_proxy()?;
        proxy
            .set_global_engine_async(engine_name)
            .await
            .map_err(|e| Error::Engine(format!("Failed to set global engine: {}", e)))?;
        Ok(())
    }

    /// Create a new InputContext via the ibus-daemon.
    ///
    /// Calls `CreateInputContext` on the daemon, which assigns a unique D-Bus
    /// object path and returns a ready-to-use [`InputContext`].
    ///
    /// Use this when your application has multiple input areas (e.g., multiple
    /// windows or text fields), each needing its own input context.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use libibus_rs::Bus;
    ///
    /// # async fn example() -> libibus_rs::Result<()> {
    /// let mut bus = Bus::new();
    /// bus.connect().await?;
    ///
    /// let ic = bus.create_input_context("my-app-window-1").await?;
    /// ic.focus_in().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_input_context(&self, client_name: &str) -> Result<InputContext> {
        let conn = self.connection()?;
        let path = {
            let proxy = self.bus_proxy()?;
            proxy
                .create_input_context(client_name)
                .await
                .map_err(|e| {
                    Error::Connection(format!(
                        "Failed to create input context '{}': {}",
                        client_name, e
                    ))
                })?
                .as_str()
                .to_string()
        };

        InputContext::with_path(conn, &path).await
    }

    /// Disconnect from the ibus-daemon.
    pub async fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            let _ = conn.close().await;
        }
        self.bus_proxy = None;
    }
}
