use libibus_rs::key::keysym;
use libibus_rs::lookup_table::LookupTable;
use libibus_rs::{EngineHandle, KeyEvent, Text};

use super::DemoEngine;

impl DemoEngine {
    pub(super) async fn process_japanese(
        &mut self,
        event: &KeyEvent,
        handle: &EngineHandle,
    ) -> bool {
        match event.keyval {
            keysym::Return | keysym::KP_Enter => {
                if !self.preedit.is_empty() {
                    self.commit_and_clear_preedit(handle).await;
                }
                true
            }

            keysym::BackSpace => {
                if self.preedit.pop().is_some() {
                    if self.preedit.is_empty() {
                        if let Err(e) = handle.hide_preedit_text().await {
                            log::warn!("hide_preedit_text failed: {}", e);
                        }
                    } else {
                        self.update_preedit(handle).await;
                    }
                }
                true
            }

            keysym::Escape => {
                self.preedit.clear();
                if let Err(e) = handle.hide_preedit_text().await {
                    log::warn!("hide_preedit_text failed: {}", e);
                }
                if let Err(e) = handle.hide_lookup_table().await {
                    log::warn!("hide_lookup_table failed: {}", e);
                }
                if let Err(e) = handle.hide_auxiliary_text().await {
                    log::warn!("hide_auxiliary_text failed: {}", e);
                }
                true
            }

            keysym::space => {
                if !self.preedit.is_empty() {
                    let mut table = LookupTable::new();
                    for c in &["候補一", "候補二", "候補三", "候補四", "候補五"] {
                        table.append_candidate(Text::new(c));
                    }
                    table.set_orientation(libibus_rs::lookup_table::LookupOrientation::Horizontal);
                    if let Err(e) = handle.update_lookup_table(table, true).await {
                        log::warn!("update_lookup_table failed: {}", e);
                    }
                }
                true
            }

            keysym::_1..=keysym::_9 => {
                let idx = (event.keyval - keysym::_1) as usize;
                if !self.preedit.is_empty() {
                    if let Err(e) = handle
                        .delete_surrounding_text(
                            -(self.preedit.len() as i32),
                            self.preedit.len() as u32,
                        )
                        .await
                    {
                        log::warn!("delete_surrounding_text failed: {}", e);
                    }
                    self.preedit.clear();
                }
                if let Err(e) = handle.hide_lookup_table().await {
                    log::warn!("hide_lookup_table failed: {}", e);
                }
                if let Err(e) = handle.commit_text(&format!("候補{}", idx + 1)).await {
                    log::warn!("commit_text failed: {}", e);
                }
                if let Err(e) = handle.hide_preedit_text().await {
                    log::warn!("hide_preedit_text failed: {}", e);
                }
                true
            }

            keysym::grave => {
                self.aux_visible = !self.aux_visible;
                if self.aux_visible {
                    if let Err(e) = handle.update_auxiliary_text("変換モード (aux)", true).await
                    {
                        log::warn!("update_auxiliary_text failed: {}", e);
                    }
                } else {
                    if let Err(e) = handle.hide_auxiliary_text().await {
                        log::warn!("hide_auxiliary_text failed: {}", e);
                    }
                }
                true
            }

            keysym::asciitilde => {
                if let Err(e) = handle.require_surrounding_text().await {
                    log::warn!("require_surrounding_text failed: {}", e);
                }
                true
            }

            keysym::a..=keysym::z => {
                let kana = event.keyval - keysym::a;
                let hira = char::from_u32(0x3042 + kana).unwrap_or('?');
                self.preedit.push(hira);
                self.update_preedit(handle).await;
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
    use libibus_rs::KeyEvent;
    use libibus_rs::engine::EngineImpl;
    use libibus_rs::key::keysym;

    #[tokio::test]
    async fn test_process_key_plain() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        let ev = KeyEvent::new(keysym::a, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert_eq!(engine.preedit, "あ");
    }

    #[tokio::test]
    async fn test_enter_commits_preedit() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        let ev = KeyEvent::new(keysym::Return, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert!(engine.preedit.is_empty());
    }

    #[tokio::test]
    async fn test_backspace() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        let ev = KeyEvent::new(keysym::BackSpace, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert_eq!(engine.preedit, "あい");
    }

    #[tokio::test]
    async fn test_escape_clears() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        let ev = KeyEvent::new(keysym::Escape, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert!(engine.preedit.is_empty());
    }

    #[tokio::test]
    async fn test_space_shows_lookup() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        let ev = KeyEvent::new(keysym::space, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
    }

    #[tokio::test]
    async fn test_number_selects_candidate() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        engine.preedit = "あいう".to_string();
        let ev = KeyEvent::new(keysym::_1, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert!(engine.preedit.is_empty());
    }

    #[tokio::test]
    async fn test_backtick_toggles_aux() {
        let Some(handle) = create_test_handle().await else {
            return;
        };
        let mut engine = DemoEngine::new();
        assert!(!engine.aux_visible);
        let ev = KeyEvent::new(keysym::grave, 0, 0);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert!(engine.aux_visible);
        assert!(engine.process_key_event(&ev, &handle).await);
        assert!(!engine.aux_visible);
    }
}
