use zbus::object_server::SignalEmitter;

use crate::engine::descriptor::Engine;
use crate::lookup_table::LookupTable;
use crate::prop::{Prop, PropList};
use crate::serializable::IBusSerializable;
/// Handle for emitting IBus engine signals.
///
/// The library provides an `EngineHandle` as a borrowed parameter in all
/// [`EngineImpl`](crate::engine::EngineImpl) callback methods once the engine's
/// D-Bus object is registered.  Use it to notify the IBus panel
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
    /// Create a new `EngineHandle` wrapping a `SignalEmitter`.
    ///
    /// # Note
    /// This constructor is primarily public to allow unit testing custom engines.
    pub fn new(signal_ctxt: SignalEmitter<'static>) -> Self {
        Self { signal_ctxt }
    }

    /// Emit the `CommitText` signal.
    pub async fn commit_text(&self, text: impl Into<crate::text::Text>) -> zbus::Result<()> {
        let text_obj = text.into();
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::commit_text(&emitter, &text_obj.to_value()).await
    }

    /// Emit the `UpdatePreeditText` signal.
    ///
    /// `mode` controls preedit behavior when focus is lost:
    /// - 0 (`PREEDIT_CLEAR`): Clear preedit text on focus out
    /// - 1 (`PREEDIT_COMMIT`): Commit preedit text on focus out
    pub async fn update_preedit_text(
        &self,
        text: impl Into<crate::text::Text>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()> {
        let text_obj = text.into();
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::update_preedit_text(&emitter, &text_obj.to_value(), cursor_pos, visible, mode).await
    }

    /// Emit the `ShowPreeditText` signal.
    pub async fn show_preedit_text(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::show_preedit_text(&emitter).await
    }

    /// Emit the `HidePreeditText` signal.
    pub async fn hide_preedit_text(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::hide_preedit_text(&emitter).await
    }

    /// Emit the `UpdateLookupTable` signal.
    pub async fn update_lookup_table(
        &self,
        lookup_table: LookupTable,
        visible: bool,
    ) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::update_lookup_table(&emitter, &lookup_table.to_value(), visible).await
    }

    /// Emit the `ShowLookupTable` signal.
    pub async fn show_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::show_lookup_table(&emitter).await
    }

    /// Emit the `HideLookupTable` signal.
    pub async fn hide_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::hide_lookup_table(&emitter).await
    }

    /// Emit the `UpdateAuxiliaryText` signal.
    pub async fn update_auxiliary_text(
        &self,
        text: impl Into<crate::text::Text>,
        visible: bool,
    ) -> zbus::Result<()> {
        let text_obj = text.into();
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::update_auxiliary_text(&emitter, &text_obj.to_value(), visible).await
    }

    /// Emit the `ShowAuxiliaryText` signal.
    pub async fn show_auxiliary_text(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::show_auxiliary_text(&emitter).await
    }

    /// Emit the `HideAuxiliaryText` signal.
    pub async fn hide_auxiliary_text(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::hide_auxiliary_text(&emitter).await
    }

    /// Emit the `RegisterProperties` signal.
    pub async fn register_properties(&self, props: PropList) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::register_properties(&emitter, &props.to_value()).await
    }

    /// Emit the `UpdateProperty` signal.
    pub async fn update_property(&self, prop: Prop) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::update_property(&emitter, &prop.to_value()).await
    }

    /// Emit the `ForwardKeyEvent` signal.
    pub async fn forward_key_event(
        &self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::forward_key_event(&emitter, keyval, keycode, state).await
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
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::delete_surrounding_text(&emitter, offset_from_cursor, nchars).await
    }

    /// Emit the `PageUpLookupTable` signal.
    pub async fn page_up_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::page_up_lookup_table(&emitter).await
    }

    /// Emit the `PageDownLookupTable` signal.
    pub async fn page_down_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::page_down_lookup_table(&emitter).await
    }

    /// Emit the `CursorUpLookupTable` signal.
    pub async fn cursor_up_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::cursor_up_lookup_table(&emitter).await
    }

    /// Emit the `CursorDownLookupTable` signal.
    pub async fn cursor_down_lookup_table(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::cursor_down_lookup_table(&emitter).await
    }

    /// Emit the `RequireSurroundingText` signal.
    ///
    /// Requests the client to send surrounding text via `SetSurroundingText`.
    /// Useful when the engine needs context around the cursor for conversion.
    pub async fn require_surrounding_text(&self) -> zbus::Result<()> {
        let emitter = SignalEmitter::clone(&self.signal_ctxt);
        Engine::require_surrounding_text(&emitter).await
    }
}
