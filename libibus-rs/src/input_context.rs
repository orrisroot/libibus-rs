use zbus::Connection;

use crate::dbus::InputContextProxy;
use crate::error::{Error, Result};

bitflags::bitflags! {
    /// IBus capabilities flags.
    ///
    /// Used with [`InputContext::set_capabilities`] to tell the engine which
    /// features the client supports.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Caps: u32 {
        /// Client supports surrounding text.
        const SURROUNDING = 1 << 0;
        /// Client supports surrounding text with selection.
        const SURROUNDING_SELECTION = 1 << 1;
        /// Client supports preedit text with attributes.
        const PREEDIT_ATTR = 1 << 2;
        /// Client supports auxiliary text.
        const AUX = 1 << 3;
        /// Client supports lookup table.
        const LOOKUP_TABLE = 1 << 4;
        /// Client supports content type hints.
        const CONTENT_HINT = 1 << 5;
        /// Client supports content purpose.
        const CONTENT_PURPOSE = 1 << 6;
        /// Client supports delete surrounding text.
        const DELETE_SURROUNDING = 1 << 7;
    }
}

/// IBus input purpose constants.
///
/// Used with [`InputContext::set_content_type`] to indicate the purpose of
/// the input field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Purpose {
    /// Normal text input.
    Normal = 0,
    /// Alpha-numeric input.
    Alpha = 1,
    /// Name input.
    Name = 2,
    /// Telephone number input.
    Number = 3,
    /// PIN input.
    Pin = 4,
    /// E-mail address input.
    Email = 5,
    /// URL input.
    Url = 6,
    /// Password input.
    Password = 7,
    /// Caption input.
    Caption = 8,
}

impl Purpose {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Normal),
            1 => Some(Self::Alpha),
            2 => Some(Self::Name),
            3 => Some(Self::Number),
            4 => Some(Self::Pin),
            5 => Some(Self::Email),
            6 => Some(Self::Url),
            7 => Some(Self::Password),
            8 => Some(Self::Caption),
            _ => None,
        }
    }
}

impl From<Purpose> for u32 {
    fn from(p: Purpose) -> Self {
        p as u32
    }
}

impl From<u32> for Purpose {
    fn from(v: u32) -> Self {
        Self::from_u32(v).unwrap_or(Self::Normal)
    }
}

bitflags::bitflags! {
    /// IBus input hint flags.
    ///
    /// Used with [`InputContext::set_content_type`] to provide hints about
    /// expected input. Combine with bitwise OR:
    ///
    /// ```rust
    /// use libibus_rs::input_context::Hint;
    /// let hints = Hint::NO_AUTO_CAPS | Hint::NO_PREDICTIVE;
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Hint: u32 {
        /// No hints.
        const NONE = 0;
        /// Do not use auto-caps.
        const NO_AUTO_CAPS = 1 << 0;
        /// Do not use auto-correction.
        const NO_AUTO_CORRECTION = 1 << 1;
        /// Do not use predictive input.
        const NO_PREDICTIVE = 1 << 2;
        /// Use lowercase.
        const LOWERCASE = 1 << 3;
        /// Use uppercase.
        const UPPERCASE = 1 << 4;
        /// Use title case.
        const TITLECASE = 1 << 5;
        /// Hide input (e.g., password).
        const HIDDEN_TEXT = 1 << 6;
        /// Do not learn new words.
        const SENSITIVE_DATA = 1 << 7;
        /// Completion is enabled.
        const COMPLETION = 1 << 8;
    }
}

/// A client-side IBus input context.
///
/// `InputContext` represents a single text-input area within an application.
/// It connects to the ibus-daemon, receives key events from the active input
/// method engine, and delivers signals such as `commit-text`, `preedit-text`,
/// and `lookup-table` to the application.
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
/// // Create an input context via the daemon (required — there is no static path)
/// let ic = bus.create_input_context("my-app").await?;
///
/// // Subscribe to commit-text signal
/// ic.connect_commit_text(|text| {
///     println!("Committed: {}", text);
/// }).await?;
///
/// // Subscribe to preedit-text signal
/// ic.connect_update_preedit_text(|text, cursor_pos, visible| {
///     println!("Preedit: {} (cursor={}, visible={})", text, cursor_pos, visible);
/// }).await?;
///
/// ic.focus_in().await?;
/// // ... process key events ...
/// ic.focus_out().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct InputContext {
    proxy: InputContextProxy<'static>,
}

/// A handle returned by [`InputContext::connect_*`] methods.
///
/// When dropped, the associated signal handler task is cancelled.
/// Keep this handle alive for as long as you want to receive the signal.
#[must_use = "dropping this handle cancels the signal subscription"]
pub struct Subscription {
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Subscription {
    /// Create a new subscription from a spawned task handle.
    pub(crate) fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    /// Manually cancel the signal handler task.
    pub fn cancel(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

impl InputContext {
    /// Create a new InputContext bound to a specific D-Bus object path.
    ///
    /// Typically used together with
    /// [`Bus::create_input_context`](crate::Bus::create_input_context) to
    /// obtain a dynamically assigned path from the ibus-daemon.
    pub async fn with_path(connection: &Connection, path: &str) -> Result<Self> {
        let object_path: zvariant::OwnedObjectPath = zvariant::ObjectPath::try_from(path)
            .map_err(|e| Error::Connection(format!("Invalid InputContext path '{}': {}", path, e)))?
            .into();

        let proxy = InputContextProxy::builder(connection)
            .destination("org.freedesktop.IBus")
            .expect("static destination is valid")
            .path(object_path)
            .map_err(|e| Error::Connection(format!("Invalid InputContext path '{}': {}", path, e)))?
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await
            .map_err(|e| {
                Error::Connection(format!("Failed to create InputContext proxy: {}", e))
            })?;
        Ok(Self { proxy })
    }

    // ==================== Methods (client → daemon/engine) ====================

    /// Send a key event to the engine for processing.
    ///
    /// Returns `true` if the engine consumed the event.
    pub async fn process_key_event(&self, keyval: u32, keycode: u32, state: u32) -> Result<bool> {
        let mut proxy = self.proxy.clone();
        proxy
            .process_key_event(keyval, keycode, state)
            .await
            .map_err(|e| Error::Engine(format!("process_key_event failed: {}", e)))
    }

    /// Notify the engine that this input context gained focus.
    pub async fn focus_in(&self) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .focus_in()
            .await
            .map_err(|e| Error::Engine(format!("focus_in failed: {}", e)))
    }

    /// Notify the engine that this input context lost focus.
    pub async fn focus_out(&self) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .focus_out()
            .await
            .map_err(|e| Error::Engine(format!("focus_out failed: {}", e)))
    }

    /// Reset the engine state (clear preedit, lookup table, etc.).
    pub async fn reset(&self) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .reset()
            .await
            .map_err(|e| Error::Engine(format!("reset failed: {}", e)))
    }

    /// Switch to a specific input method engine by name.
    pub async fn set_engine(&self, engine_name: &str) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .set_engine(engine_name)
            .await
            .map_err(|e| Error::Engine(format!("set_engine failed: {}", e)))
    }

    /// Get the name of the current engine.
    pub async fn get_engine(&self) -> Result<String> {
        let proxy = self.proxy.clone();
        proxy
            .get_engine()
            .await
            .map_err(|e| Error::Engine(format!("get_engine failed: {}", e)))
    }

    /// Report the cursor location and size to the engine.
    ///
    /// Coordinates are relative to the root window.
    pub async fn set_cursor_location(&self, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .set_cursor_location(x, y, w, h)
            .await
            .map_err(|e| Error::Engine(format!("set_cursor_location failed: {}", e)))
    }

    /// Set the capabilities supported by this input context.
    ///
    /// Use [`Caps`] constants to build the capability bitmask:
    ///
    /// ```rust,no_run
    /// use libibus_rs::input_context::Caps;
    /// # async fn example(ic: &libibus_rs::InputContext) -> libibus_rs::Result<()> {
    /// let caps = Caps::SURROUNDING | Caps::PREEDIT_ATTR | Caps::LOOKUP_TABLE;
    /// ic.set_capabilities(caps).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_capabilities(&self, caps: Caps) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .set_capabilities(caps.bits())
            .await
            .map_err(|e| Error::Engine(format!("set_capabilities failed: {}", e)))
    }

    /// Provide surrounding text around the cursor to the engine.
    pub async fn set_surrounding_text(
        &self,
        text: &str,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .set_surrounding_text(text, cursor_pos, anchor_pos)
            .await
            .map_err(|e| Error::Engine(format!("set_surrounding_text failed: {}", e)))
    }

    /// Set the content type (hints + purpose) of the input field.
    ///
    /// Use [`Hint`] and [`Purpose`] for type-safe constants.
    pub async fn set_content_type(&self, hints: u32, purpose: u32) -> Result<()> {
        let mut proxy = self.proxy.clone();
        proxy
            .set_content_type(hints, purpose)
            .await
            .map_err(|e| Error::Engine(format!("set_content_type failed: {}", e)))
    }

    // ==================== Signal subscriptions (engine → client) ================

    /// Subscribe to the `commit-text` signal.
    ///
    /// Called when the engine commits final text to the input field.
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_commit_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(String) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_commit_text()
            .await
            .map_err(|e| Error::Connection(format!("Failed to receive commit_text: {}", e)))?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.text.to_string());
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `update-preedit-text` signal.
    ///
    /// Called when the engine updates the preedit (composition) text.
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_update_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(String, u32, bool) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_update_preedit_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive update_preedit_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.text.to_string(), args.cursor_pos, args.visible);
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `show-preedit-text` signal.
    pub async fn connect_show_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_show_preedit_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive show_preedit_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `hide-preedit-text` signal.
    pub async fn connect_hide_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_hide_preedit_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive hide_preedit_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `update-auxiliary-text` signal.
    ///
    /// Called when the engine updates auxiliary text (e.g., status line).
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_update_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(String, bool) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_update_auxiliary_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive update_auxiliary_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.text.to_string(), args.visible);
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `show-auxiliary-text` signal.
    pub async fn connect_show_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_show_auxiliary_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive show_auxiliary_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `hide-auxiliary-text` signal.
    pub async fn connect_hide_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_hide_auxiliary_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive hide_auxiliary_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `update-lookup-table` signal.
    ///
    /// Called when the engine updates the candidate lookup table.
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_update_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(crate::LookupTable, bool) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_update_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive update_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.lookup_table.clone(), args.visible);
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `show-lookup-table` signal.
    pub async fn connect_show_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_show_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive show_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `hide-lookup-table` signal.
    pub async fn connect_hide_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_hide_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive hide_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `page-up-lookup-table` signal.
    pub async fn connect_page_up_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_page_up_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive page_up_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `page-down-lookup-table` signal.
    pub async fn connect_page_down_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_page_down_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive page_down_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `cursor-up-lookup-table` signal.
    pub async fn connect_cursor_up_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_cursor_up_lookup_table().await.map_err(|e| {
            Error::Connection(format!("Failed to receive cursor_up_lookup_table: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `cursor-down-lookup-table` signal.
    pub async fn connect_cursor_down_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_cursor_down_lookup_table()
            .await
            .map_err(|e| {
                Error::Connection(format!("Failed to receive cursor_down_lookup_table: {}", e))
            })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `forward-key-event` signal.
    ///
    /// Called when the engine forwards an unconsumed key event back to the client.
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_forward_key_event<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(u32, u32, u32) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_forward_key_event().await.map_err(|e| {
            Error::Connection(format!("Failed to receive forward_key_event: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.keyval, args.keycode, args.state);
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `delete-surrounding-text` signal.
    ///
    /// Called when the engine requests deletion of surrounding text.
    /// `offset_from_cursor` is the starting offset relative to the cursor
    /// (negative = before cursor), and `nchars` is the number of characters to delete.
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_delete_surrounding_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn(i32, u32) + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy.receive_delete_surrounding_text().await.map_err(|e| {
            Error::Connection(format!("Failed to receive delete_surrounding_text: {}", e))
        })?;

        let handle = crate::signal::spawn_handler(stream, move |signal| {
            if let Ok(args) = signal.args() {
                callback(args.offset_from_cursor, args.nchars);
            }
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `disabled` signal.
    ///
    /// Called when the engine is disabled.
    pub async fn connect_disabled<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_disabled()
            .await
            .map_err(|e| Error::Connection(format!("Failed to receive disabled: {}", e)))?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }

    /// Subscribe to the `require-surrounding-text` signal.
    ///
    /// Called when the engine requests surrounding text from the client.
    /// The client should respond by calling [`set_surrounding_text`](Self::set_surrounding_text).
    /// Returns a [`Subscription`] that cancels the handler when dropped.
    pub async fn connect_require_surrounding_text<F>(&self, callback: F) -> Result<Subscription>
    where
        F: Fn() + Send + 'static,
    {
        let proxy = self.proxy.clone();
        let stream = proxy
            .receive_require_surrounding_text()
            .await
            .map_err(|e| {
                Error::Connection(format!("Failed to receive require_surrounding_text: {}", e))
            })?;

        let handle = crate::signal::spawn_handler(stream, move |_signal| {
            callback();
        });

        Ok(Subscription::new(handle))
    }
}
