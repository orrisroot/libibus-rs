use zbus::Connection;

use crate::dbus::PanelProxy;
use crate::error::{Error, Result};

/// Client for the IBus panel (`org.freedesktop.IBus.Panel`).
///
/// Provides methods for interacting with the IBus panel (focus notification)
/// and subscribing to `engine_activate` signals.
///
/// # Example
///
/// ```rust,no_run
/// use libibus_rs::Panel;
///
/// # async fn example(conn: &zbus::Connection) -> libibus_rs::Result<()> {
/// let panel = Panel::new(conn).await?;
/// panel.focus_in().await?;
/// # Ok(())
/// # }
/// ```
pub struct Panel {
    proxy: PanelProxy<'static>,
}

impl Panel {
    /// Create a new Panel client.
    pub async fn new(connection: &Connection) -> Result<Self> {
        let proxy = PanelProxy::new(connection)
            .await
            .map_err(|e| Error::Connection(format!("Failed to create Panel proxy: {}", e)))?;
        Ok(Self { proxy })
    }

    /// Notify the panel that the input context gained focus.
    pub async fn focus_in(&self) -> Result<()> {
        let proxy = self.proxy.clone();
        proxy
            .focus_in()
            .await
            .map_err(|e| Error::Connection(format!("Panel focus_in failed: {}", e)))
    }

    /// Notify the panel that the input context lost focus.
    pub async fn focus_out(&self) -> Result<()> {
        let proxy = self.proxy.clone();
        proxy
            .focus_out()
            .await
            .map_err(|e| Error::Connection(format!("Panel focus_out failed: {}", e)))
    }

    /// Reset the panel state.
    pub async fn reset(&self) -> Result<()> {
        let proxy = self.proxy.clone();
        proxy
            .reset()
            .await
            .map_err(|e| Error::Connection(format!("Panel reset failed: {}", e)))
    }

    /// Subscribe to `engine_activate` signals.
    ///
    /// The callback is invoked in a background task each time an engine is
    /// activated.
    pub async fn connect_engine_activate<F>(&self, callback: F) -> Result<crate::Subscription>
    where
        F: Fn(String) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_engine_activate()
            .await
            .map_err(|e| Error::Connection(format!("Failed to receive engine_activate: {}", e)))?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.engine_name.to_string());
            }
        });

        Ok(crate::Subscription::new(handle))
    }
}
