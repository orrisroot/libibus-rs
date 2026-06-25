use zbus::{interface, object_server::SignalEmitter};

use crate::engine::descriptor::Engine;
use crate::key::KeyEvent;

#[interface(name = "org.freedesktop.IBus.Engine")]
impl Engine {
    async fn process_key_event(
        &self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::fdo::Result<bool> {
        let event = KeyEvent::new(keyval, keycode, state);
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        let consumed = inner.process_key_event(&event, &handle).await;
        Ok(consumed)
    }

    async fn focus_in(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.focus_in(&handle).await;
        Ok(())
    }

    async fn focus_out(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.focus_out(&handle).await;
        Ok(())
    }

    async fn reset(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.reset(&handle).await;
        Ok(())
    }

    async fn enable(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.enable(&handle).await;
        Ok(())
    }

    async fn disable(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.disable(&handle).await;
        Ok(())
    }

    async fn set_cursor_location(&self, x: i32, y: i32, w: i32, h: i32) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.set_cursor_location(x, y, w, h, &handle).await;
        Ok(())
    }

    async fn set_content_type(&self, purpose: u32, hints: u32) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.set_content_type(hints, purpose, &handle).await;
        Ok(())
    }

    async fn set_surrounding_text(
        &self,
        text: zvariant::OwnedValue,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        // unwrap_serializable strips the IBusText class-name wrapper and returns
        // the inner structure. For IBusText (class_name, dict, text), it returns
        // the text string (fields[2]).
        let text_str = crate::serializable::unwrap_serializable(&text.into(), "IBusText")
            .ok()
            .and_then(|v| {
                if let zvariant::Value::Str(s) = &v {
                    Some(s.to_string())
                } else if let zvariant::Value::Structure(s) = &v {
                    let fields = s.fields();
                    // After stripping class_name and dict, remaining fields are the IBusText fields
                    if !fields.is_empty() {
                        fields[0].clone().try_into().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_default();
        inner
            .set_surrounding_text(&text_str, cursor_pos, anchor_pos, &handle)
            .await;
        Ok(())
    }

    async fn set_engine_name(&self, name: String) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.set_engine_name(&name, &handle).await;
        Ok(())
    }

    async fn page_up(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.page_up(&handle).await;
        Ok(())
    }

    async fn page_down(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.page_down(&handle).await;
        Ok(())
    }

    async fn cursor_up(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.cursor_up(&handle).await;
        Ok(())
    }

    async fn cursor_down(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.cursor_down(&handle).await;
        Ok(())
    }

    async fn candidate_clicked(
        &self,
        index: u32,
        button: u32,
        state: u32,
    ) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.candidate_clicked(index, button, state, &handle).await;
        Ok(())
    }

    async fn property_activate(&self, prop_name: String, prop_state: u32) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner
            .property_activate(&prop_name, prop_state, &handle)
            .await;
        Ok(())
    }

    async fn property_show(&self, prop_name: String) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.property_show(&prop_name, &handle).await;
        Ok(())
    }

    async fn property_hide(&self, prop_name: String) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.property_hide(&prop_name, &handle).await;
        Ok(())
    }

    async fn destroy(&self) -> zbus::fdo::Result<()> {
        let handle = self.handle.clone();
        let mut inner = self.inner.lock().await;
        inner.destroy(&handle).await;
        Ok(())
    }

    // ==================== Signals ====================

    #[zbus(signal)]
    pub(crate) async fn commit_text(
        ctxt: &SignalEmitter<'_>,
        text: &zvariant::Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn update_preedit_text(
        ctxt: &SignalEmitter<'_>,
        text: &zvariant::Value<'_>,
        cursor_pos: u32,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn show_preedit_text(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn hide_preedit_text(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn update_lookup_table(
        ctxt: &SignalEmitter<'_>,
        lookup_table: &zvariant::Value<'_>,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn show_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn hide_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn update_auxiliary_text(
        ctxt: &SignalEmitter<'_>,
        text: &zvariant::Value<'_>,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn show_auxiliary_text(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn hide_auxiliary_text(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn register_properties(
        ctxt: &SignalEmitter<'_>,
        props: &zvariant::Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn update_property(
        ctxt: &SignalEmitter<'_>,
        prop: &zvariant::Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn forward_key_event(
        ctxt: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn delete_surrounding_text(
        ctxt: &SignalEmitter<'_>,
        offset_from_cursor: i32,
        nchars: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn page_up_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn page_down_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn cursor_up_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn cursor_down_lookup_table(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub(crate) async fn require_surrounding_text(ctxt: &SignalEmitter<'_>) -> zbus::Result<()>;
}
