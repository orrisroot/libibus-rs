use zbus::Connection;
use zvariant::OwnedValue;

use crate::dbus::ConfigProxy;
use crate::error::{Error, Result};

/// Client for the IBus configuration daemon (`org.freedesktop.IBus.Config`).
///
/// Provides access to ibus-daemon configuration values.  Supports reading and
/// writing typed configuration values (string, i32, bool, f64) and listening
/// for changes.
///
/// # Example
///
/// ```rust,no_run
/// use libibus_rs::Config;
///
/// # async fn example(conn: &zbus::Connection) -> libibus_rs::Result<()> {
/// let config = Config::new(conn).await?;
/// let val = config.get_value("general", "name").await?;
/// # Ok(())
/// # }
/// ```
pub struct Config {
    proxy: ConfigProxy<'static>,
}

impl Config {
    /// Create a new Config client.
    pub async fn new(connection: &Connection) -> Result<Self> {
        let proxy = ConfigProxy::new(connection)
            .await
            .map_err(|e| Error::Connection(format!("Failed to create Config proxy: {}", e)))?;
        Ok(Self { proxy })
    }

    /// Get a raw configuration value.
    pub async fn get_value(&self, section: &str, name: &str) -> Result<OwnedValue> {
        let proxy = self.proxy.clone();
        proxy
            .get_value(section, name)
            .await
            .map_err(|e| Error::Connection(format!("Config get_value failed: {}", e)))
    }

    /// Get a string configuration value.
    pub async fn get_string(&self, section: &str, name: &str) -> Result<String> {
        let value = self.get_value(section, name).await?;
        use zvariant::Value;
        match value.into() {
            Value::Str(s) => Ok(s.to_string()),
            _ => Err(Error::InvalidArgument("Value is not a string".into())),
        }
    }

    /// Get an `i32` configuration value.
    pub async fn get_i32(&self, section: &str, name: &str) -> Result<i32> {
        let value = self.get_value(section, name).await?;
        use zvariant::Value;
        match value.into() {
            Value::I32(v) => Ok(v),
            Value::U32(v) => Ok(v as i32),
            _ => Err(Error::InvalidArgument("Value is not an int32".into())),
        }
    }

    /// Get a `bool` configuration value.
    pub async fn get_bool(&self, section: &str, name: &str) -> Result<bool> {
        let value = self.get_value(section, name).await?;
        use zvariant::Value;
        match value.into() {
            Value::Bool(v) => Ok(v),
            _ => Err(Error::InvalidArgument("Value is not a bool".into())),
        }
    }

    /// Get an `f64` configuration value.
    pub async fn get_f64(&self, section: &str, name: &str) -> Result<f64> {
        let value = self.get_value(section, name).await?;
        use zvariant::Value;
        match value.into() {
            Value::F64(v) => Ok(v),
            Value::I32(v) => Ok(v as f64),
            Value::U32(v) => Ok(v as f64),
            _ => Err(Error::InvalidArgument("Value is not a double".into())),
        }
    }

    /// Set a configuration value.
    pub async fn set_value(
        &self,
        section: &str,
        name: &str,
        value: &zvariant::Value<'_>,
    ) -> Result<()> {
        let proxy = self.proxy.clone();
        proxy
            .set_value(section, name, value)
            .await
            .map_err(|e| Error::Connection(format!("Config set_value failed: {}", e)))
    }

    /// Unset a configuration value.
    pub async fn unset(&self, section: &str, name: &str) -> Result<()> {
        let proxy = self.proxy.clone();
        proxy
            .unset(section, name)
            .await
            .map_err(|e| Error::Connection(format!("Config unset failed: {}", e)))
    }

    /// Get all key-value pairs in a section.
    pub async fn get_values(&self, section: &str) -> Result<Vec<(String, OwnedValue)>> {
        let proxy = self.proxy.clone();
        proxy
            .get_values(section)
            .await
            .map_err(|e| Error::Connection(format!("Config get_values failed: {}", e)))
    }

    /// Subscribe to `value_changed` signals.
    ///
    /// The callback is invoked in a background task each time a configuration
    /// value changes.
    pub async fn connect_value_changed<F>(&self, callback: F) -> Result<crate::Subscription>
    where
        F: Fn(String, String, OwnedValue) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_value_changed()
            .await
            .map_err(|e| Error::Connection(format!("Failed to receive value_changed: {}", e)))?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.section.to_string(), args.name.to_string(), args.value);
            }
        });

        Ok(crate::Subscription::new(handle))
    }
}
