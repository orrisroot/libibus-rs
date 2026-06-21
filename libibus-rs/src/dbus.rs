use zbus::proxy;

use crate::component::Component;
use crate::lookup_table::LookupTable;
use zvariant::OwnedObjectPath;

#[proxy(
    interface = "org.freedesktop.IBus",
    default_service = "org.freedesktop.IBus",
    default_path = "/org/freedesktop/IBus"
)]
pub trait IBus {
    fn hello(&mut self) -> zbus::Result<OwnedObjectPath>;

    fn create_input_context(&mut self, client_name: &str) -> zbus::Result<OwnedObjectPath>;

    fn register_component(&mut self, component: Component) -> zbus::Result<()>;

    fn get_address(&mut self) -> zbus::Result<String>;

    fn set_global_engine_async(&mut self, engine_name: &str) -> zbus::Result<()>;

    fn current_input_context(&self) -> zbus::Result<OwnedObjectPath>;

    fn name_owner(&self) -> zbus::Result<String>;

    fn register_properties(&mut self, props: &zvariant::Value<'_>) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.IBus.Factory",
    default_service = "org.freedesktop.IBus"
)]
pub trait FactoryProxy {
    fn create_engine(&mut self, engine_name: &str) -> zbus::Result<OwnedObjectPath>;

    fn destroy_engine(&mut self, engine_name: &str) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.IBus.Panel",
    default_service = "org.freedesktop.IBus",
    default_path = "/org/freedesktop/IBus/Panel"
)]
pub trait Panel {
    fn focus_in(&mut self) -> zbus::Result<()>;
    fn focus_out(&mut self) -> zbus::Result<()>;
    fn reset(&mut self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn engine_activate(&self, engine_name: &str) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.IBus.Config",
    default_service = "org.freedesktop.IBus",
    default_path = "/org/freedesktop/IBus/Config"
)]
pub trait Config {
    fn get_value(&mut self, section: &str, name: &str) -> zbus::Result<zvariant::OwnedValue>;

    fn set_value(
        &mut self,
        section: &str,
        name: &str,
        value: &zvariant::Value<'_>,
    ) -> zbus::Result<()>;

    fn unset(&mut self, section: &str, name: &str) -> zbus::Result<()>;

    fn get_values(&mut self, section: &str) -> zbus::Result<Vec<(String, zvariant::OwnedValue)>>;

    #[zbus(signal)]
    fn value_changed(
        &self,
        section: &str,
        name: &str,
        value: zvariant::OwnedValue,
    ) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.IBus.InputContext",
    default_service = "org.freedesktop.IBus"
)]
pub trait InputContext {
    fn focus_in(&mut self) -> zbus::Result<()>;
    fn focus_out(&mut self) -> zbus::Result<()>;
    fn reset(&mut self) -> zbus::Result<()>;
    fn set_engine(&mut self, engine_name: &str) -> zbus::Result<()>;
    fn get_engine(&self) -> zbus::Result<String>;
    fn set_cursor_location(&mut self, x: i32, y: i32, w: i32, h: i32) -> zbus::Result<()>;
    fn set_capabilities(&mut self, caps: u32) -> zbus::Result<()>;
    fn set_surrounding_text(
        &mut self,
        text: &str,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> zbus::Result<()>;
    fn set_content_type(&mut self, hints: u32, purpose: u32) -> zbus::Result<()>;
    fn process_key_event(&mut self, keyval: u32, keycode: u32, state: u32) -> zbus::Result<bool>;

    // --- Signals emitted by the engine (received by the client app) ---

    #[zbus(signal)]
    fn commit_text(&self, text: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn update_preedit_text(&self, text: &str, cursor_pos: u32, visible: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    fn show_preedit_text(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn hide_preedit_text(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn update_auxiliary_text(&self, text: &str, visible: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    fn show_auxiliary_text(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn hide_auxiliary_text(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn update_lookup_table(&self, lookup_table: LookupTable, visible: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    fn show_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn hide_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn page_up_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn page_down_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn cursor_up_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn cursor_down_lookup_table(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn forward_key_event(&self, keyval: u32, keycode: u32, state: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn delete_surrounding_text(&self, offset_from_cursor: i32, nchars: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn disabled(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn require_surrounding_text(&self) -> zbus::Result<()>;
}
