use zbus::{interface, object_server::SignalEmitter};

use crate::engine::descriptor::Engine;
use crate::key::KeyEvent;

#[interface(name = "org.freedesktop.IBus.Engine")]
impl Engine {
    async fn process_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::fdo::Result<bool> {
        let event = KeyEvent::new(keyval, keycode, state);
        let mut inner = self.inner.lock().await;
        Ok(inner.process_key_event(&event, &self.handle).await)
    }

    async fn focus_in(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.focus_in(&self.handle).await;
        Ok(())
    }

    async fn focus_out(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.focus_out(&self.handle).await;
        Ok(())
    }

    async fn reset(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.reset(&self.handle).await;
        Ok(())
    }

    async fn enable(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.enable(&self.handle).await;
        Ok(())
    }

    async fn disable(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.disable(&self.handle).await;
        Ok(())
    }

    async fn set_cursor_location(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .set_cursor_location(x, y, w, h, &self.handle)
            .await;
        Ok(())
    }

    async fn set_content_type(&mut self, hints: u32, purpose: u32) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .set_content_type(hints, purpose, &self.handle)
            .await;
        Ok(())
    }

    async fn set_surrounding_text(
        &mut self,
        text: String,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .set_surrounding_text(&text, cursor_pos, anchor_pos, &self.handle)
            .await;
        Ok(())
    }

    async fn set_engine_name(&mut self, name: String) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .set_engine_name(&name, &self.handle)
            .await;
        Ok(())
    }

    async fn page_up(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.page_up(&self.handle).await;
        Ok(())
    }

    async fn page_down(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.page_down(&self.handle).await;
        Ok(())
    }

    async fn cursor_up(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.cursor_up(&self.handle).await;
        Ok(())
    }

    async fn cursor_down(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.cursor_down(&self.handle).await;
        Ok(())
    }

    async fn candidate_clicked(
        &mut self,
        index: u32,
        button: u32,
        state: u32,
    ) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .candidate_clicked(index, button, state, &self.handle)
            .await;
        Ok(())
    }

    async fn property_activate(
        &mut self,
        prop_name: String,
        prop_state: u32,
    ) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .property_activate(&prop_name, prop_state, &self.handle)
            .await;
        Ok(())
    }

    async fn property_show(&mut self, prop_name: String) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .property_show(&prop_name, &self.handle)
            .await;
        Ok(())
    }

    async fn property_hide(&mut self, prop_name: String) -> zbus::fdo::Result<()> {
        self.inner
            .lock()
            .await
            .property_hide(&prop_name, &self.handle)
            .await;
        Ok(())
    }

    async fn destroy(&mut self) -> zbus::fdo::Result<()> {
        self.inner.lock().await.destroy(&self.handle).await;
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
