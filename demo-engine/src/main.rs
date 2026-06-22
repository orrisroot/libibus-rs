use std::time::Duration;

use libibus_rs::engine::EngineImpl;
use libibus_rs::error::Result;
use libibus_rs::factory::FactoryImpl;
use libibus_rs::key::keysym;
use libibus_rs::lookup_table::LookupTable;
use libibus_rs::prop::{Prop, PropList, PropState};
use libibus_rs::{Bus, Component, EngineDesc, EngineHandle, KeyEvent, Text};

#[cfg(test)]
use libibus_rs::ModifierType;

/// A demo engine that illustrates the full IBus engine API surface.
///
/// - Type `a`–`z` to accumulate preedit text (displayed as Japanese hiragana).
/// - Press **Space** to show a lookup table of mock conversion candidates.
/// - Press **1**–**9** to select a candidate (commits it via `delete_surrounding_text` + `commit_text`).
/// - Press **Enter** to commit the current preedit text.
/// - Press **Backspace** to delete the last character.
/// - Press **Escape** to cancel.
/// - Press **`** (backtick) to toggle auxiliary text display.
/// - Press **~** (tilde) to request surrounding text from the client.
///
/// Properties:
/// - "mode" toggle — demonstrates property activation handling.
pub struct DemoEngine {
    pub preedit: String,
    pub aux_visible: bool,
    pub mode_checked: bool,
}

impl DemoEngine {
    pub fn new() -> Self {
        Self {
            preedit: String::new(),
            aux_visible: false,
            mode_checked: false,
        }
    }

    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Check whether a key event should be filtered out (not processed by the engine).
///
/// Returns `true` if the event should be **ignored** (e.g., key release, Ctrl, Alt).
pub fn should_ignore_key(event: &KeyEvent) -> bool {
    event.modifiers().is_release() || event.modifiers().is_control() || event.modifiers().is_alt()
}

#[async_trait::async_trait]
impl EngineImpl for DemoEngine {
    async fn process_key_event(&mut self, event: &KeyEvent, handle: &EngineHandle) -> bool {
        if should_ignore_key(event) {
            return false;
        }

        match event.keyval {
            keysym::Return | keysym::KP_Enter => {
                if !self.preedit.is_empty() {
                    handle.commit_text(&self.preedit).await.unwrap_or(());
                    handle.hide_preedit_text().await.unwrap_or(());
                    handle.hide_lookup_table().await.unwrap_or(());
                    self.preedit.clear();
                }
                true
            }

            keysym::BackSpace => {
                if self.preedit.pop().is_some() {
                    if self.preedit.is_empty() {
                        handle.hide_preedit_text().await.unwrap_or(());
                    } else {
                        handle
                            .update_preedit_text(&self.preedit, self.preedit.len() as u32, true)
                            .await
                            .unwrap_or(());
                    }
                }
                true
            }

            keysym::Escape => {
                self.preedit.clear();
                handle.hide_preedit_text().await.unwrap_or(());
                handle.hide_lookup_table().await.unwrap_or(());
                handle.hide_auxiliary_text().await.unwrap_or(());
                true
            }

            keysym::space => {
                if !self.preedit.is_empty() {
                    let mut table = LookupTable::new();
                    for c in &["候補一", "候補二", "候補三", "候補四", "候補五"] {
                        table.append_candidate(Text::new(c));
                    }
                    table.set_orientation(libibus_rs::lookup_table::LookupOrientation::Horizontal);
                    handle.update_lookup_table(table, true).await.unwrap_or(());
                }
                true
            }

            keysym::_1..=keysym::_9 => {
                let idx = (event.keyval - keysym::_1) as usize;
                // Demonstrate delete_surrounding_text: delete preedit length chars before cursor,
                // then commit the selected candidate.
                if !self.preedit.is_empty() {
                    handle
                        .delete_surrounding_text(
                            -(self.preedit.len() as i32),
                            self.preedit.len() as u32,
                        )
                        .await
                        .unwrap_or(());
                    self.preedit.clear();
                }
                handle.hide_lookup_table().await.unwrap_or(());
                handle
                    .commit_text(&format!("候補{}", idx + 1))
                    .await
                    .unwrap_or(());
                handle.hide_preedit_text().await.unwrap_or(());
                true
            }

            // Toggle auxiliary text
            keysym::grave => {
                self.aux_visible = !self.aux_visible;
                if self.aux_visible {
                    handle
                        .update_auxiliary_text("変換モード (aux)", true)
                        .await
                        .unwrap_or(());
                } else {
                    handle.hide_auxiliary_text().await.unwrap_or(());
                }
                true
            }

            // Request surrounding text from the client
            keysym::asciitilde => {
                handle.require_surrounding_text().await.unwrap_or(());
                true
            }

            keysym::a..=keysym::z => {
                let kana = (event.keyval - keysym::a) as u32;
                let hira = char::from_u32(0x3042 + kana).unwrap_or('?');
                self.preedit.push(hira);
                handle
                    .update_preedit_text(&self.preedit, self.preedit.len() as u32, true)
                    .await
                    .unwrap_or(());
                true
            }

            _ => false,
        }
    }

    async fn focus_in(&mut self, handle: &EngineHandle) {
        let _ = handle.show_preedit_text().await;

        // Register properties on focus
        let mut props = PropList::new();
        let mut mode_prop = Prop::toggle("mode", "変換");
        mode_prop.set_tooltip("変換モードを切り替え");
        mode_prop.set_icon("ibus-mode-indicator");
        mode_prop.set_state(if self.mode_checked {
            PropState::Checked
        } else {
            PropState::Unchecked
        });
        props.append(mode_prop);

        let separator = Prop::separator();
        props.append(separator);

        let mut info_prop = Prop::new("info", "ℹ️");
        info_prop.set_tooltip("libibus-rs demo engine");
        props.append(info_prop);

        let _ = handle.register_properties(props).await;
    }

    async fn focus_out(&mut self, handle: &EngineHandle) {
        let _ = handle.hide_preedit_text().await;
        let _ = handle.hide_lookup_table().await;
        let _ = handle.hide_auxiliary_text().await;
    }

    async fn reset(&mut self, handle: &EngineHandle) {
        self.preedit.clear();
        let _ = handle.hide_preedit_text().await;
        let _ = handle.hide_lookup_table().await;
    }

    /// Handle property activation from the panel.
    async fn property_activate(&mut self, prop_name: &str, prop_state: u32, handle: &EngineHandle) {
        match prop_name {
            "mode" => {
                self.mode_checked = prop_state == PropState::Checked as u32;
                let mut prop = Prop::toggle("mode", "変換");
                prop.set_state(prop_state.into());
                let _ = handle.update_property(prop).await;
            }
            "info" => {
                // Show auxiliary text as a response to info button
                self.aux_visible = true;
                let _ = handle
                    .update_auxiliary_text(
                        "libibus-rs demo — https://github.com/orrisroot/libibus-rs",
                        true,
                    )
                    .await;
            }
            _ => {}
        }
    }

    /// Handle candidate clicked from the panel.
    async fn candidate_clicked(&mut self, index: u32, _button: u32, _state: u32, handle: &EngineHandle) {
        let candidate = format!("候補{}", index + 1);
        let _ = handle.commit_text(&candidate).await;
        let _ = handle.hide_lookup_table().await;
        let _ = handle.hide_preedit_text().await;
        self.preedit.clear();
    }

    /// Handle lookup table navigation from the panel.
    async fn page_up(&mut self, _handle: &EngineHandle) {
        // In a real engine, update the lookup table's cursor position
    }

    async fn page_down(&mut self, _handle: &EngineHandle) {
        // In a real engine, update the lookup table's cursor position
    }

    async fn cursor_up(&mut self, _handle: &EngineHandle) {}

    async fn cursor_down(&mut self, _handle: &EngineHandle) {}

    /// Handle engine destruction.
    async fn destroy(&mut self, _handle: &EngineHandle) {
        self.preedit.clear();
    }
}

/// Factory that creates `DemoEngine` instances.
struct DemoFactory;

#[async_trait::async_trait]
impl FactoryImpl for DemoFactory {
    async fn create_engine(&mut self, engine_name: &str) -> Result<Box<dyn EngineImpl>> {
        println!("Factory asked to create engine: {}", engine_name);
        Ok(Box::new(DemoEngine {
            preedit: String::new(),
            aux_visible: false,
            mode_checked: false,
        }))
    }

    async fn destroy_engine(&mut self, engine_name: &str) -> Result<()> {
        println!("Factory asked to destroy engine: {}", engine_name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut bus = Bus::new();
    bus.connect().await?;
    let conn = bus.connection().unwrap().clone();

    let mut component_desc = Component::new(
        "com.github.orrisroot.libibus-rs.Demo",
        "libibus-rs demo engine",
        "0.1.0",
        "MIT",
        "Yoshihiro OKUMURA <orrisroot@gmail.com>",
        "https://github.com/orrisroot/libibus-rs",
        "/usr/libexec/libibus-rs-engine-demo",
    );

    let mut engine_desc = EngineDesc::new(
        "libibus-rs-demo",
        "libibus-rs Demo",
        "Demonstration engine for libibus-rs",
        "ja",
    );
    engine_desc
        .set_license("MIT")
        .set_author("Yoshihiro OKUMURA")
        .set_symbol("あ");

    bus.register_component(component_desc.add_engine(engine_desc))
        .await?;

    libibus_rs::factory::register(&conn, Box::new(DemoFactory)).await?;

    println!("Demo engine registered. Waiting for D-Bus requests...");
    println!("Press Ctrl+C to stop.");

    // Keep process alive to serve D-Bus requests
    loop {
        tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== should_ignore_key tests ====================

    #[test]
    fn test_should_ignore_release() {
        let ev = KeyEvent::new(keysym::a, 0, ModifierType::RELEASE.bits());
        assert!(should_ignore_key(&ev));
    }

    #[test]
    fn test_should_ignore_control() {
        let ev = KeyEvent::new(keysym::a, 0, ModifierType::CONTROL.bits());
        assert!(should_ignore_key(&ev));
    }

    #[test]
    fn test_should_ignore_alt() {
        let ev = KeyEvent::new(keysym::a, 0, ModifierType::MOD1.bits());
        assert!(should_ignore_key(&ev));
    }

    #[test]
    fn test_should_not_ignore_plain_key() {
        let ev = KeyEvent::new(keysym::a, 0, 0);
        assert!(!should_ignore_key(&ev));
    }

    #[test]
    fn test_should_not_ignore_shift() {
        let ev = KeyEvent::new(keysym::a, 0, ModifierType::SHIFT.bits());
        assert!(!should_ignore_key(&ev));
    }

    #[test]
    fn test_should_ignore_control_and_shift() {
        let ev = KeyEvent::new(
            keysym::a,
            0,
            ModifierType::CONTROL.bits() | ModifierType::SHIFT.bits(),
        );
        assert!(should_ignore_key(&ev));
    }

    async fn create_test_handle() -> Option<EngineHandle> {
        if let Ok(conn) = zbus::Connection::session().await {
            let path = zvariant::ObjectPath::try_from("/org/freedesktop/IBus/Engine/Test").unwrap();
            if let Ok(emitter) = zbus::object_server::SignalEmitter::new(&conn, path) {
                return Some(EngineHandle::new(emitter.into_owned()));
            }
        }
        None
    }

    // ==================== DemoEngine state tests ====================

    #[test]
    fn test_demo_engine_new_state() {
        let engine = DemoEngine::new();
        assert!(engine.preedit.is_empty());
        assert!(!engine.aux_visible);
        assert!(!engine.mode_checked);
    }

    #[tokio::test]
    async fn test_process_key_plain_returns_true() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        let ev = KeyEvent::new(keysym::a, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert_eq!(engine.preedit, "あ");
    }

    #[tokio::test]
    async fn test_process_key_ignore_release() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        let ev = KeyEvent::new(keysym::a, 0, ModifierType::RELEASE.bits());
        assert!(!engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_process_key_ignore_ctrl() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        let ev = KeyEvent::new(keysym::c, 0, ModifierType::CONTROL.bits());
        assert!(!engine.process_key_event(&ev, &handle).await);
    }

    // ==================== Property activation state tests ====================

    #[tokio::test]
    async fn test_property_activate_mode_toggle() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        assert!(!engine.mode_checked);

        engine
            .property_activate("mode", PropState::Checked as u32, &handle)
            .await;
        assert!(engine.mode_checked);

        engine
            .property_activate("mode", PropState::Unchecked as u32, &handle)
            .await;
        assert!(!engine.mode_checked);
    }

    #[tokio::test]
    async fn test_property_activate_info_shows_aux() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        assert!(!engine.aux_visible);

        engine.property_activate("info", 0, &handle).await;
        assert!(engine.aux_visible);
    }

    #[tokio::test]
    async fn test_property_activate_unknown_ignored() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        let initial_mode = engine.mode_checked;
        let initial_aux = engine.aux_visible;

        engine.property_activate("unknown_prop", 0, &handle).await;
        assert_eq!(engine.mode_checked, initial_mode);
        assert_eq!(engine.aux_visible, initial_aux);
    }

    // ==================== Candidate clicked state tests ====================

    #[tokio::test]
    async fn test_candidate_clicked_clears_preedit() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();

        engine.candidate_clicked(0, 1, 0, &handle).await;
        assert_eq!(engine.preedit, "");
    }

    // ==================== Destroy state tests ====================

    #[tokio::test]
    async fn test_destroy_clears_state() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        engine.aux_visible = true;
        engine.mode_checked = true;

        engine.destroy(&handle).await;
        assert!(engine.preedit.is_empty());
    }

    // ==================== Reset state tests ====================

    #[tokio::test]
    async fn test_reset_clears_preedit() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "テスト".to_string();

        engine.reset(&handle).await;
        assert!(engine.preedit.is_empty());
    }

    // ==================== Factory tests ====================

    #[tokio::test]
    async fn test_factory_create_engine() {
        let mut factory = DemoFactory;
        let _engine = factory.create_engine("test-engine").await.unwrap();
        // Engine should be created successfully (no panic)
    }

    #[tokio::test]
    async fn test_factory_destroy_engine() {
        let mut factory = DemoFactory;
        factory.destroy_engine("test-engine").await.unwrap();
        // Should not panic
    }
}
