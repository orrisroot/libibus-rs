use zbus::object_server::SignalEmitter;

use crate::engine::descriptor::Engine;
use crate::lookup_table::LookupTable;
use crate::prop::{Prop, PropList};

/// Handle for emitting IBus engine signals.
///
/// The daemon gives you an `EngineHandle` via
/// [`EngineImpl::set_handle`](crate::engine::EngineImpl::set_handle) after
/// the engine's D-Bus object is registered.  Use it to notify the IBus panel
/// of state changes:
///
/// | Method | Effect |
/// |---|---|
/// | [`commit_text`](Self::commit_text) | Output final text |
/// | [`update_preedit_text`](Self::update_preedit_text) | Show in‑line composition |
/// | [`update_lookup_table`](Self::update_lookup_table) | Show conversion candidates |
/// | [`register_properties`](Self::register_properties) | Add UI buttons / menus |
///
/// All methods wrap the corresponding `org.freedesktop.IBus.Engine` D-Bus
/// signals.
#[derive(Clone)]
pub struct EngineHandle {
    signal_ctxt: SignalEmitter<'static>,
}

impl EngineHandle {
    pub(crate) fn new(signal_ctxt: SignalEmitter<'static>) -> Self {
        Self { signal_ctxt }
    }

    /// Emit the `CommitText` signal.
    pub async fn commit_text(&self, text: &str) -> zbus::Result<()> {
        Engine::commit_text(&self.signal_ctxt, text).await
    }

    /// Emit the `UpdatePreeditText` signal.
    pub async fn update_preedit_text(
        &self,
        text: &str,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()> {
        Engine::update_preedit_text(&self.signal_ctxt, text, cursor_pos, visible).await
    }

    /// Emit the `ShowPreeditText` signal.
    pub async fn show_preedit_text(&self) -> zbus::Result<()> {
        Engine::show_preedit_text(&self.signal_ctxt).await
    }

    /// Emit the `HidePreeditText` signal.
    pub async fn hide_preedit_text(&self) -> zbus::Result<()> {
        Engine::hide_preedit_text(&self.signal_ctxt).await
    }

    /// Emit the `UpdateLookupTable` signal.
    pub async fn update_lookup_table(
        &self,
        lookup_table: LookupTable,
        visible: bool,
    ) -> zbus::Result<()> {
        Engine::update_lookup_table(&self.signal_ctxt, lookup_table, visible).await
    }

    /// Emit the `ShowLookupTable` signal.
    pub async fn show_lookup_table(&self) -> zbus::Result<()> {
        Engine::show_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `HideLookupTable` signal.
    pub async fn hide_lookup_table(&self) -> zbus::Result<()> {
        Engine::hide_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `UpdateAuxiliaryText` signal.
    pub async fn update_auxiliary_text(&self, text: &str, visible: bool) -> zbus::Result<()> {
        Engine::update_auxiliary_text(&self.signal_ctxt, text, visible).await
    }

    /// Emit the `ShowAuxiliaryText` signal.
    pub async fn show_auxiliary_text(&self) -> zbus::Result<()> {
        Engine::show_auxiliary_text(&self.signal_ctxt).await
    }

    /// Emit the `HideAuxiliaryText` signal.
    pub async fn hide_auxiliary_text(&self) -> zbus::Result<()> {
        Engine::hide_auxiliary_text(&self.signal_ctxt).await
    }

    /// Emit the `RegisterProperties` signal.
    pub async fn register_properties(&self, props: PropList) -> zbus::Result<()> {
        Engine::register_properties(&self.signal_ctxt, props).await
    }

    /// Emit the `UpdateProperty` signal.
    pub async fn update_property(&self, prop: Prop) -> zbus::Result<()> {
        Engine::update_property(&self.signal_ctxt, prop).await
    }

    /// Emit the `ForwardKeyEvent` signal.
    pub async fn forward_key_event(
        &self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()> {
        Engine::forward_key_event(&self.signal_ctxt, keyval, keycode, state).await
    }

    /// Emit the `DeleteSurroundingText` signal.
    ///
    /// Requests deletion of surrounding text.  `offset_from_cursor` is the
    /// starting offset relative to the cursor (negative = before cursor),
    /// and `nchars` is the number of characters to delete.
    pub async fn delete_surrounding_text(
        &self,
        offset_from_cursor: i32,
        nchars: u32,
    ) -> zbus::Result<()> {
        Engine::delete_surrounding_text(&self.signal_ctxt, offset_from_cursor, nchars).await
    }

    /// Emit the `PageUpLookupTable` signal.
    pub async fn page_up_lookup_table(&self) -> zbus::Result<()> {
        Engine::page_up_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `PageDownLookupTable` signal.
    pub async fn page_down_lookup_table(&self) -> zbus::Result<()> {
        Engine::page_down_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `CursorUpLookupTable` signal.
    pub async fn cursor_up_lookup_table(&self) -> zbus::Result<()> {
        Engine::cursor_up_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `CursorDownLookupTable` signal.
    pub async fn cursor_down_lookup_table(&self) -> zbus::Result<()> {
        Engine::cursor_down_lookup_table(&self.signal_ctxt).await
    }

    /// Emit the `RequireSurroundingText` signal.
    ///
    /// Requests the client to send surrounding text via `SetSurroundingText`.
    /// Useful when the engine needs context around the cursor for conversion.
    pub async fn require_surrounding_text(&self) -> zbus::Result<()> {
        Engine::require_surrounding_text(&self.signal_ctxt).await
    }
}
