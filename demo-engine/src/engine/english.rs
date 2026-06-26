use libibus_rs::key::keysym;
use libibus_rs::{EngineHandle, KeyEvent};

use super::DemoEngine;

impl DemoEngine {
    pub(super) async fn process_english(
        &mut self,
        event: &KeyEvent,
        handle: &EngineHandle,
    ) -> bool {
        match event.keyval {
            keysym::a..=keysym::z | keysym::A..=keysym::Z => {
                let c = char::from_u32(event.keyval).unwrap_or('?');
                if let Err(e) = handle.commit_text(&c.to_string()).await {
                    log::warn!("commit_text failed: {}", e);
                }
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::DemoEngine;
    use super::super::create_test_handle;
    use libibus_rs::engine::EngineImpl;
    use libibus_rs::key::keysym;
    use libibus_rs::{KeyEvent, ModifierType};

    #[tokio::test]
    async fn test_process_key_letter_returns_true() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.mode = super::super::InputMode::English;
        let ev = KeyEvent::new(keysym::a, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_process_key_uppercase_returns_true() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.mode = super::super::InputMode::English;
        let ev = KeyEvent::new(keysym::A, 0, ModifierType::SHIFT.bits());
        assert!(engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_process_key_number_returns_false() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.mode = super::super::InputMode::English;
        let ev = KeyEvent::new(keysym::_1, 0, 0);
        assert!(!engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_process_key_space_returns_false() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.mode = super::super::InputMode::English;
        let ev = KeyEvent::new(keysym::space, 0, 0);
        assert!(!engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_process_key_enter_returns_false() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.mode = super::super::InputMode::English;
        let ev = KeyEvent::new(keysym::Return, 0, 0);
        assert!(!engine.process_key_event(&ev, &handle).await);
    }
}
