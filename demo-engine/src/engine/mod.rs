pub mod english;
pub mod japanese;

use libibus_rs::engine::EngineImpl;
use libibus_rs::error::Result;
use libibus_rs::factory::FactoryImpl;
use libibus_rs::key::ModifierType;
use libibus_rs::prop::{Prop, PropList, PropState, PropType};
use libibus_rs::{EngineHandle, KeyEvent, LookupTable};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Japanese,
    English,
}

impl InputMode {}

pub struct DemoEngine {
    pub preedit: String,
    pub aux_visible: bool,
    pub mode: InputMode,
    pub lookup_table: Option<LookupTable>,
}

impl Default for DemoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoEngine {
    pub fn new() -> Self {
        Self {
            preedit: String::new(),
            aux_visible: false,
            mode: InputMode::Japanese,
            lookup_table: None,
        }
    }

    fn mode_menu_prop(&self) -> Prop {
        let mut menu = Prop::new("mode", "Input Mode");
        menu.prop_type = PropType::Menu as u32;
        menu.set_icon(match self.mode {
            InputMode::Japanese => "ibus-keyboard-jp",
            InputMode::English => "ibus-keyboard-us",
        });

        let japanese_prop = {
            let mut p = Prop::radio("mode_japanese", "Japanese");
            p.set_symbol("あ");
            p.set_tooltip("日本語入力モード");
            p.set_state(if self.mode == InputMode::Japanese {
                PropState::Checked
            } else {
                PropState::Unchecked
            });
            p
        };

        let english_prop = {
            let mut p = Prop::radio("mode_english", "English");
            p.set_symbol("EN");
            p.set_tooltip("English input mode");
            p.set_state(if self.mode == InputMode::English {
                PropState::Checked
            } else {
                PropState::Unchecked
            });
            p
        };

        let mut sub_props = PropList::new();
        sub_props.append(japanese_prop);
        sub_props.append(english_prop);
        menu.set_sub_props(sub_props);
        menu
    }

    pub(super) async fn commit_and_clear_preedit(&mut self, handle: &EngineHandle) {
        let text = std::mem::take(&mut self.preedit);
        if !text.is_empty() {
            if let Err(e) = handle.commit_text(&text).await {
                log::warn!("commit_text failed: {}", e);
            }
        }
        if let Err(e) = handle.hide_preedit_text().await {
            log::warn!("hide_preedit_text failed: {}", e);
        }
        if let Err(e) = handle.hide_lookup_table().await {
            log::warn!("hide_lookup_table failed: {}", e);
        }
    }

    pub(super) async fn update_preedit(&self, handle: &EngineHandle) {
        if let Err(e) = handle
            .update_preedit_text(&self.preedit, self.preedit.len() as u32, true, 0)
            .await
        {
            log::warn!("update_preedit_text failed: {}", e);
        }
    }
}

pub fn should_ignore_key(event: &KeyEvent) -> bool {
    event.modifiers().is_release() || event.modifiers().is_control() || event.modifiers().is_alt()
}

#[async_trait::async_trait]
impl EngineImpl for DemoEngine {
    async fn process_key_event(&mut self, event: &KeyEvent, handle: &EngineHandle) -> bool {
        if event.modifiers() == ModifierType::CONTROL && event.keyval == libibus_rs::key::keysym::space {
            self.mode = match self.mode {
                InputMode::Japanese => InputMode::English,
                InputMode::English => InputMode::Japanese,
            };
            self.preedit.clear();

            if let Err(e) = handle.hide_preedit_text().await {
                log::warn!("hide_preedit_text failed: {}", e);
            }
            if let Err(e) = handle.hide_lookup_table().await {
                log::warn!("hide_lookup_table failed: {}", e);
            }

            let mut props = PropList::new();
            props.append({
                let mut p = Prop::new("setup", "Setup");
                p.set_visible(true);
                p
            });
            props.append(self.mode_menu_prop());
            if let Err(e) = handle.register_properties(props).await {
                log::warn!("register_properties failed: {}", e);
            }

            if self.mode == InputMode::Japanese {
                if let Err(e) = handle.show_preedit_text().await {
                    log::warn!("show_preedit_text failed: {}", e);
                }
            }
            return true;
        }

        if should_ignore_key(event) {
            return false;
        }

        match self.mode {
            InputMode::Japanese => self.process_japanese(event, handle).await,
            InputMode::English => self.process_english(event, handle).await,
        }
    }

    async fn focus_in(&mut self, handle: &EngineHandle) {
        if self.mode == InputMode::Japanese {
            if let Err(e) = handle.show_preedit_text().await {
                log::warn!("show_preedit_text failed: {}", e);
            }
        }

        let mut props = PropList::new();
        props.append({
            let mut p = Prop::new("setup", "Setup");
            p.set_visible(true);
            p
        });
        props.append(self.mode_menu_prop());
        handle.register_properties(props).await.ok();
    }

    async fn focus_out(&mut self, handle: &EngineHandle) {
        if let Err(e) = handle.hide_preedit_text().await {
            log::warn!("hide_preedit_text failed: {}", e);
        }
        if let Err(e) = handle.hide_lookup_table().await {
            log::warn!("hide_lookup_table failed: {}", e);
        }
        if let Err(e) = handle.hide_auxiliary_text().await {
            log::warn!("hide_auxiliary_text failed: {}", e);
        }
    }

    async fn reset(&mut self, handle: &EngineHandle) {
        self.preedit.clear();
        self.lookup_table.take();
        if let Err(e) = handle.hide_preedit_text().await {
            log::warn!("hide_preedit_text failed: {}", e);
        }
        if let Err(e) = handle.hide_lookup_table().await {
            log::warn!("hide_lookup_table failed: {}", e);
        }
    }

    async fn property_activate(
        &mut self,
        prop_name: &str,
        _prop_state: u32,
        handle: &EngineHandle,
    ) {
        match prop_name {
            "mode_japanese" => {
                if self.mode == InputMode::Japanese {
                    return;
                }
                self.mode = InputMode::Japanese;
                self.preedit.clear();

                if let Err(e) = handle.hide_preedit_text().await {
                    log::warn!("hide_preedit_text failed: {}", e);
                }
                if let Err(e) = handle.hide_lookup_table().await {
                    log::warn!("hide_lookup_table failed: {}", e);
                }

                let mut props = PropList::new();
                props.append({
                    let mut p = Prop::new("setup", "Setup");
                    p.set_visible(true);
                    p
                });
                props.append(self.mode_menu_prop());
                if let Err(e) = handle.register_properties(props).await {
                    log::warn!("register_properties failed: {}", e);
                }
            }
            "mode_english" => {
                if self.mode == InputMode::English {
                    return;
                }
                self.mode = InputMode::English;
                self.preedit.clear();

                if let Err(e) = handle.hide_preedit_text().await {
                    log::warn!("hide_preedit_text failed: {}", e);
                }
                if let Err(e) = handle.hide_lookup_table().await {
                    log::warn!("hide_lookup_table failed: {}", e);
                }

                let mut props = PropList::new();
                props.append({
                    let mut p = Prop::new("setup", "Setup");
                    p.set_visible(true);
                    p
                });
                props.append(self.mode_menu_prop());
                if let Err(e) = handle.register_properties(props).await {
                    log::warn!("register_properties failed: {}", e);
                }
            }
            "info" => {
                self.aux_visible = true;
                if let Err(e) = handle
                    .update_auxiliary_text(
                        "libibus-rs demo — https://github.com/orrisroot/libibus-rs",
                        true,
                    )
                    .await
                {
                    log::warn!("update_auxiliary_text failed: {}", e);
                }
            }
            _ => {}
        }
    }

    async fn candidate_clicked(
        &mut self,
        index: u32,
        _button: u32,
        _state: u32,
        handle: &EngineHandle,
    ) {
        self.preedit.clear();
        let candidate = format!("候補{}", index + 1);
        if let Err(e) = handle.commit_text(&candidate).await {
            log::warn!("commit_text failed: {}", e);
        }
        if let Err(e) = handle.hide_lookup_table().await {
            log::warn!("hide_lookup_table failed: {}", e);
        }
        if let Err(e) = handle.hide_preedit_text().await {
            log::warn!("hide_preedit_text failed: {}", e);
        }
    }

    async fn page_up(&mut self, _handle: &EngineHandle) {}
    async fn page_down(&mut self, _handle: &EngineHandle) {}
    async fn cursor_up(&mut self, _handle: &EngineHandle) {}
    async fn cursor_down(&mut self, _handle: &EngineHandle) {}

    async fn destroy(&mut self, _handle: &EngineHandle) {
        self.preedit.clear();
        self.lookup_table.take();
    }
}

pub struct DemoFactory;

#[async_trait::async_trait]
impl FactoryImpl for DemoFactory {
    async fn create_engine(&mut self, engine_name: &str) -> Result<Box<dyn EngineImpl>> {
        println!("Factory asked to create engine: {}", engine_name);
        Ok(Box::new(DemoEngine::new()))
    }

    async fn destroy_engine(&mut self, engine_name: &str) -> Result<()> {
        println!("Factory asked to destroy engine: {}", engine_name);
        Ok(())
    }
}

#[cfg(test)]
pub(crate) async fn create_test_handle() -> Option<EngineHandle> {
    if let Ok(conn) = zbus::Connection::session().await {
        let path = zvariant::ObjectPath::try_from("/org/freedesktop/IBus/Engine/Test").unwrap();
        if let Ok(emitter) = zbus::object_server::SignalEmitter::new(&conn, path) {
            return Some(EngineHandle::new(emitter.into_owned()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use libibus_rs::key::keysym;

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

    #[test]
    fn test_demo_engine_new_state() {
        let engine = DemoEngine::new();
        assert!(engine.preedit.is_empty());
        assert!(!engine.aux_visible);
        assert_eq!(engine.mode, InputMode::Japanese);
        assert!(engine.lookup_table.is_none());
    }

    #[tokio::test]
    async fn test_property_activate_mode_japanese() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        assert_eq!(engine.mode, InputMode::Japanese);

        engine.property_activate("mode_japanese", 0, &handle).await;
        assert_eq!(engine.mode, InputMode::Japanese);

        engine.mode = InputMode::English;
        engine.property_activate("mode_japanese", 0, &handle).await;
        assert_eq!(engine.mode, InputMode::Japanese);
    }

    #[tokio::test]
    async fn test_property_activate_mode_english() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        assert_eq!(engine.mode, InputMode::Japanese);

        engine.property_activate("mode_english", 0, &handle).await;
        assert_eq!(engine.mode, InputMode::English);

        engine.property_activate("mode_english", 0, &handle).await;
        assert_eq!(engine.mode, InputMode::English);
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
        let initial = engine.mode;
        engine.property_activate("unknown_prop", 0, &handle).await;
        assert_eq!(engine.mode, initial);
    }

    #[tokio::test]
    async fn test_mode_switch_clears_preedit() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        engine.property_activate("mode_english", 0, &handle).await;
        assert_eq!(engine.mode, InputMode::English);
        assert!(engine.preedit.is_empty());
    }

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

    #[tokio::test]
    async fn test_destroy_clears_state() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        engine.destroy(&handle).await;
        assert!(engine.preedit.is_empty());
    }

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

    #[tokio::test]
    async fn test_factory_create_engine() {
        let mut factory = DemoFactory;
        let _engine = factory.create_engine("libibus-rs-demo").await.unwrap();
    }

    #[tokio::test]
    async fn test_factory_destroy_engine() {
        let mut factory = DemoFactory;
        factory.destroy_engine("libibus-rs-demo").await.unwrap();
    }
}
