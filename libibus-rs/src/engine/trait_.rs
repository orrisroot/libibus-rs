use async_trait::async_trait;

use crate::engine::handle::EngineHandle;
use crate::key::KeyEvent;

/// User-implementable trait for an IBus input method engine.
///
/// Implement this trait and pass it to
/// [`factory::register`](crate::factory::register) via a
/// [`FactoryImpl`](crate::factory::FactoryImpl) to create a working IBus
/// engine.
///
/// All methods have default no-op implementations so you only need to override
/// the ones you care about.
///
/// # Lifecycle
///
/// 1. [`enable`](EngineImpl::enable) / [`disable`](EngineImpl::disable) are
///    called when the engine is activated or deactivated by the user.
/// 2. [`process_key_event`](EngineImpl::process_key_event) is called for every
///    key press when the engine is active, receiving an [`EngineHandle`] context.
#[async_trait]
pub trait EngineImpl: Send {
    /// Process a key event. Return `true` if the event was handled (consumed).
    ///
    /// The default implementation returns `false`, indicating the event was
    /// not handled.
    async fn process_key_event(&mut self, _event: &KeyEvent, _handle: &EngineHandle) -> bool {
        false
    }

    /// The input context gained focus.
    async fn focus_in(&mut self, _handle: &EngineHandle) {}

    /// The input context lost focus.
    async fn focus_out(&mut self, _handle: &EngineHandle) {}

    /// Reset the engine state.
    async fn reset(&mut self, _handle: &EngineHandle) {}

    /// The engine was enabled (activated by the user).
    async fn enable(&mut self, _handle: &EngineHandle) {}

    /// The engine was disabled.
    async fn disable(&mut self, _handle: &EngineHandle) {}

    /// The cursor location changed.
    async fn set_cursor_location(
        &mut self,
        _x: i32,
        _y: i32,
        _w: i32,
        _h: i32,
        _handle: &EngineHandle,
    ) {
    }

    /// The content type of the focused input field changed.
    async fn set_content_type(&mut self, _hints: u32, _purpose: u32, _handle: &EngineHandle) {}

    /// Surrounding text around the cursor was provided.
    async fn set_surrounding_text(
        &mut self,
        _text: &str,
        _cursor_pos: u32,
        _anchor_pos: u32,
        _handle: &EngineHandle,
    ) {
    }

    /// The engine name was set by the daemon.
    async fn set_engine_name(&mut self, _name: &str, _handle: &EngineHandle) {}

    /// Lookup table page up requested by the panel.
    async fn page_up(&mut self, _handle: &EngineHandle) {}

    /// Lookup table page down requested by the panel.
    async fn page_down(&mut self, _handle: &EngineHandle) {}

    /// Lookup table cursor up requested by the panel.
    async fn cursor_up(&mut self, _handle: &EngineHandle) {}

    /// Lookup table cursor down requested by the panel.
    async fn cursor_down(&mut self, _handle: &EngineHandle) {}

    /// A candidate in the lookup table was clicked.
    ///
    /// `index` is the candidate index, `button` is the mouse button number,
    /// and `state` is the modifier state.
    async fn candidate_clicked(
        &mut self,
        _index: u32,
        _button: u32,
        _state: u32,
        _handle: &EngineHandle,
    ) {
    }

    /// A property (button/menu item) was activated by the user.
    ///
    /// `prop_name` is the property key, `prop_state` is the new state value.
    async fn property_activate(
        &mut self,
        _prop_name: &str,
        _prop_state: u32,
        _handle: &EngineHandle,
    ) {
    }

    /// Show a specific property in the panel.
    async fn property_show(&mut self, _prop_name: &str, _handle: &EngineHandle) {}

    /// Hide a specific property in the panel.
    async fn property_hide(&mut self, _prop_name: &str, _handle: &EngineHandle) {}

    /// The engine is being destroyed.
    async fn destroy(&mut self, _handle: &EngineHandle) {}
}
