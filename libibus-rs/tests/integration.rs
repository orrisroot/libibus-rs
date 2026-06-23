use libibus_rs::engine::EngineImpl;
use libibus_rs::factory::FactoryImpl;
use libibus_rs::input_context::{Caps, Hint, Purpose};
use libibus_rs::key::keysym;
use libibus_rs::lookup_table::LookupTable;
use libibus_rs::prop::{Prop, PropList, PropState};
use libibus_rs::{
    Attr, AttrList, AttrType, Bus, Component, EngineDesc, KeyEvent, ModifierType, Text,
};

/// Return `true` if an ibus-daemon appears to be running on this system.
fn has_ibus_daemon() -> bool {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".into());
    let display_num = display
        .trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0");
    let config_dir = std::env::var("HOME")
        .map(|h| format!("{}/.config/ibus/bus", h))
        .unwrap_or_else(|_| "/tmp/ibus".into());

    if std::env::var("IBUS_ADDRESS").is_ok() {
        return true;
    }

    let entries = match std::fs::read_dir(&config_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    let suffix = format!("-unix-{}", display_num);
    let suffix_short = format!("-{}", display_num);
    for entry in entries.flatten() {
        let name = entry.file_name();
        if let Some(s) = name.to_str() {
            if s.starts_with(&format!("{}-unix-", display_num))
                || s.ends_with(&suffix)
                || s.ends_with(&suffix_short)
            {
                return true;
            }
        }
    }
    false
}

// ==================== Bus tests ====================

#[tokio::test]
async fn test_bus_connect_and_disconnect() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    assert!(!bus.is_connected());

    bus.connect().await.expect("connect to ibus-daemon");
    assert!(bus.is_connected());

    let addr = bus.get_address().await.expect("get address");
    assert!(!addr.is_empty(), "address should not be empty");

    bus.disconnect().await;
    assert!(!bus.is_connected());
}

#[tokio::test]
async fn test_bus_connection_reuse() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    bus.connect().await.expect("first connect");

    let _conn = bus.connection().expect("have connection");
    let addr1 = bus.get_address().await.expect("address");
    let addr2 = bus.get_address().await.expect("address again");
    assert_eq!(addr1, addr2);

    // Connection should still be valid after multiple proxy clones
    let proxy = bus.bus_proxy().expect("bus proxy");
    drop(proxy);
    let _ = bus.bus_proxy().expect("bus proxy after drop");

    bus.disconnect().await;
}

#[tokio::test]
async fn test_bus_create_input_context() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    bus.connect().await.expect("connect");

    let ic = bus
        .create_input_context("test-client")
        .await
        .expect("create input context");

    // Should be able to call methods on it
    ic.reset().await.expect("reset");
    ic.set_content_type(Hint::NONE.bits(), Purpose::Normal.to_u32())
        .await
        .expect("set content type");

    bus.disconnect().await;
}

// ==================== Component tests ====================

#[test]
fn test_component_creation() {
    let mut comp = Component::new(
        "test-comp",
        "Test Component",
        "1.0.0",
        "MIT",
        "Test Author",
        "https://example.com",
        "/usr/bin/test-engine",
    );

    assert_eq!(comp.name, "test-comp");
    assert_eq!(comp.engines.len(), 0);

    let mut engine = EngineDesc::new("test-eng", "Test Engine", "A test engine", "ja");
    engine
        .set_license("MIT")
        .set_author("Author")
        .set_symbol("あ")
        .set_version("2.0");

    comp.add_engine(engine);
    assert_eq!(comp.engines.len(), 1);
    assert_eq!(comp.engines[0].name, "test-eng");
    assert_eq!(comp.engines[0].symbol, "あ");
    assert_eq!(comp.engines[0].version, "2.0");
}

#[test]
fn test_component_xml_generation() {
    let mut comp = Component::new(
        "test-xml",
        "Test XML",
        "0.1.0",
        "MIT",
        "Author",
        "https://example.com",
        "/usr/bin/test",
    );
    comp.add_engine(EngineDesc::new("eng", "Engine", "Desc", "ja"));

    let xml = libibus_rs::xml::component_to_xml(&comp);
    assert!(xml.starts_with("<?xml"));
    assert!(xml.contains("<component>"));
    assert!(xml.contains("<name>test-xml</name>"));
    assert!(xml.contains("<engine>"));
    assert!(xml.contains("<name>eng</name>"));
}

// ==================== Text & Attr tests ====================

#[test]
fn test_text_creation() {
    let t = Text::new("hello");
    assert_eq!(t.text, "hello");
    assert_eq!(t.cursor_pos, 5);
    assert_eq!(t.len(), 5);
    assert!(!t.is_empty());
}

#[test]
fn test_text_append_and_clear() {
    let mut t = Text::new("hello");
    t.append(" world");
    assert_eq!(t.text, "hello world");
    assert_eq!(t.cursor_pos, 11);

    t.clear();
    assert!(t.is_empty());
    assert_eq!(t.cursor_pos, 0);
}

#[test]
fn test_text_from_str() {
    let t: Text = "from str".into();
    assert_eq!(t.text, "from str");
}

#[test]
fn test_attr_list() {
    let mut attrs = AttrList::new();
    attrs.append(Attr::underline(1, 0, 5));
    attrs.append(Attr::foreground(0xFF0000FF, 0, 5));
    assert_eq!(attrs.len(), 2);
    assert!(!attrs.is_empty());
}

#[test]
fn test_attr_type_conversion() {
    assert_eq!(AttrType::Underline.to_u32(), 0);
    assert_eq!(AttrType::Foreground.to_u32(), 1);
    assert_eq!(AttrType::Background.to_u32(), 2);
    assert!(AttrType::from_u32(0) == Some(AttrType::Underline));
    assert!(AttrType::from_u32(99).is_none());
}

// ==================== LookupTable tests ====================

#[test]
fn test_lookup_table_basic() {
    let mut table = LookupTable::new();
    assert!(table.is_empty());
    assert_eq!(table.number_of_candidates(), 0);

    table.append_candidate(Text::new("候補1"));
    table.append_candidate(Text::new("候補2"));
    table.append_candidate(Text::new("候補3"));

    assert_eq!(table.number_of_candidates(), 3);
    assert!(!table.is_empty());
}

#[test]
fn test_lookup_table_cursor_navigation() {
    let mut table = LookupTable::new();
    for i in 1..=10 {
        table.append_candidate(Text::new(&format!("c{}", i)));
    }

    table.set_cursor_pos(0);
    assert_eq!(table.cursor_pos(), 0);

    assert!(table.cursor_down());
    assert_eq!(table.cursor_pos(), 1);

    assert!(table.cursor_up());
    assert_eq!(table.cursor_pos(), 0);
}

#[test]
fn test_lookup_table_paging() {
    let mut table = LookupTable::new();
    for i in 1..=15 {
        table.append_candidate(Text::new(&format!("c{}", i)));
    }
    table.set_page_size(5);

    // 15 candidates, page_size=5 → 3 pages (0, 5, 10)
    assert!(table.page_down()); // → 5
    assert!(table.page_down()); // → 10
    assert!(!table.page_down()); // already on last page

    assert!(table.page_up()); // → 5
    assert!(table.page_up()); // → 0
    assert!(!table.page_up()); // already on first page
}

#[test]
fn test_lookup_table_current_page() {
    let mut table = LookupTable::new();
    for i in 1..=10 {
        table.append_candidate(Text::new(&format!("c{}", i)));
    }
    table.set_page_size(3);

    let page = table.current_page();
    assert_eq!(page.len(), 3);
    assert_eq!(page[0].text, "c1");
}

// ==================== Prop tests ====================

#[test]
fn test_prop_toggle() {
    let mut prop = Prop::toggle("mode", "変換");
    assert!(prop.is_toggle());
    assert!(!prop.is_checked());

    prop.set_checked(true);
    assert!(prop.is_checked());
    assert_eq!(prop.state, PropState::Checked as u32);

    prop.set_checked(false);
    assert!(!prop.is_checked());
    assert_eq!(prop.state, PropState::Unchecked as u32);
}

#[test]
fn test_prop_separator() {
    let prop = Prop::separator();
    assert!(prop.is_separator());
    assert!(!prop.sensitive);
}

#[test]
fn test_prop_list() {
    let mut pl = PropList::new();
    assert!(pl.is_empty());

    pl.append(Prop::new("key1", "Label1"));
    pl.append(Prop::toggle("key2", "Toggle"));
    assert_eq!(pl.len(), 2);

    let found = pl.get("key1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().label.text, "Label1");

    assert!(pl.get("nonexistent").is_none());
}

#[test]
fn test_prop_list_update() {
    let mut pl = PropList::new();
    let prop = Prop::toggle("mode", "Off");
    pl.append(prop);

    let mut updated = Prop::new("mode", "On");
    updated.set_state(PropState::Checked);
    pl.update_property(&updated);

    let found = pl.get("mode").unwrap();
    assert_eq!(found.state, PropState::Checked as u32);
    assert_eq!(found.label.text, "On");
}

// ==================== KeyEvent tests ====================

#[test]
fn test_key_event_creation() {
    let ev = KeyEvent::new(0x0061, 0x26, 0);
    assert_eq!(ev.keyval, 0x0061);
    assert_eq!(ev.keycode, 0x26);
    assert!(!ev.modifiers().is_shift());
}

#[test]
fn test_key_event_modifiers() {
    let ev = KeyEvent::new(
        0x0061,
        0x26,
        ModifierType::SHIFT.bits() | ModifierType::CONTROL.bits(),
    );
    assert!(ev.modifiers().is_shift());
    assert!(ev.modifiers().is_control());
    assert!(!ev.modifiers().is_alt());
}

#[test]
fn test_key_event_from_tuple() {
    let ev: KeyEvent = (0x0061, 0x26, 0).into();
    assert_eq!(ev.keyval, 0x0061);
}

#[test]
fn test_key_event_is_modifier() {
    let shift = KeyEvent::new(keysym::Shift_L, 0, 0);
    assert!(shift.is_modifier());

    let ctrl = KeyEvent::new(keysym::Control_L, 0, 0);
    assert!(ctrl.is_modifier());

    let a = KeyEvent::new(keysym::a, 0, 0);
    assert!(!a.is_modifier());
}

// ==================== ModifierType tests ====================

#[test]
fn test_modifier_type_flags() {
    let m = ModifierType::SHIFT | ModifierType::MOD4;
    assert!(m.contains(ModifierType::SHIFT));
    assert!(m.contains(ModifierType::MOD4));
    assert!(!m.contains(ModifierType::CONTROL));

    assert!(m.is_shift());
    assert!(m.is_super());
    assert!(!m.is_control());
}

// ==================== Caps tests ====================

#[test]
fn test_caps_bitwise() {
    let caps = Caps::SURROUNDING | Caps::PREEDIT_ATTR | Caps::LOOKUP_TABLE;
    assert!(caps.contains(Caps::SURROUNDING));
    assert!(caps.contains(Caps::PREEDIT_ATTR));
    assert!(caps.contains(Caps::LOOKUP_TABLE));
}

#[test]
fn test_caps_none() {
    assert_eq!(Caps::empty().bits(), 0);
}

// ==================== Engine trait default impl tests ====================

#[tokio::test]
async fn test_engine_trait_defaults() {
    struct NoOpEngine;

    #[async_trait::async_trait]
    impl EngineImpl for NoOpEngine {
        async fn process_key_event(
            &mut self,
            _event: &KeyEvent,
            _handle: &libibus_rs::EngineHandle,
        ) -> bool {
            false
        }
        // All other methods use default implementations
    }

    let mut bus = libibus_rs::Bus::new();
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }
    bus.connect().await.expect("connect to ibus-daemon");
    let connection = bus.connection().unwrap();
    let path = zvariant::ObjectPath::try_from("/org/freedesktop/IBus/Engine/Test").unwrap();
    let signal_ctxt = zbus::object_server::SignalEmitter::new(connection, path).unwrap();
    let handle = libibus_rs::EngineHandle::new(signal_ctxt.into_owned());

    let mut engine = NoOpEngine;

    // All default methods should not panic
    assert!(
        !engine
            .process_key_event(&KeyEvent::new(0x0061, 0, 0), &handle)
            .await
    );
    engine.focus_in(&handle).await;
    engine.focus_out(&handle).await;
    engine.reset(&handle).await;
    engine.enable(&handle).await;
    engine.disable(&handle).await;
    engine.set_cursor_location(0, 0, 100, 20, &handle).await;
    engine.set_content_type(0, 0, &handle).await;
    engine.set_surrounding_text("test", 4, 4, &handle).await;
    engine.set_engine_name("test", &handle).await;
    engine.page_up(&handle).await;
    engine.page_down(&handle).await;
    engine.cursor_up(&handle).await;
    engine.cursor_down(&handle).await;
    engine.candidate_clicked(0, 1, 0, &handle).await;
    engine.property_activate("mode", 1, &handle).await;
    engine.property_show("mode", &handle).await;
    engine.property_hide("mode", &handle).await;
    engine.destroy(&handle).await;

    bus.disconnect().await;
}

// ==================== EngineHandle test ====================

// EngineHandle requires a real D-Bus connection to construct,
// so we only verify that the type exists and is Clone.
#[test]
fn test_engine_handle_type() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<libibus_rs::EngineHandle>();
}

// ==================== InputContext with_path test ====================

#[tokio::test]
async fn test_input_context_with_path() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    bus.connect().await.expect("connect");

    // Create via daemon
    let ic = bus
        .create_input_context("test-path-client")
        .await
        .expect("create input context");

    // Test set_capabilities
    ic.set_capabilities(Caps::SURROUNDING | Caps::PREEDIT_ATTR)
        .await
        .expect("set capabilities");

    // Test set_cursor_location
    ic.set_cursor_location(10, 20, 100, 20)
        .await
        .expect("set cursor location");

    // Test set_surrounding_text
    ic.set_surrounding_text("hello world", 5, 5)
        .await
        .expect("set surrounding text");

    // Test set_engine
    if let Err(e) = ic.set_engine("xkb:us::eng").await {
        // We only allow setting to fail if the environment has use-global-engine enabled.
        // Any other connection or signature mismatch errors must fail the test.
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("use-global-engine") || err_msg.contains("Cannot set engines"),
            "set_engine failed with unexpected error: {:?}",
            e
        );
        eprintln!("Info: set_engine was bypassed because use-global-engine is active.");
    }

    // Test get_engine
    let engine_name = ic.get_engine().await.expect("get engine");
    assert!(!engine_name.is_empty() || engine_name == "xkb:us::eng");

    // Test Clone
    let ic2 = ic.clone();
    ic2.reset().await.expect("reset on cloned context");

    bus.disconnect().await;
}

// ==================== Subscription test ====================

#[tokio::test]
async fn test_subscription_cancellation() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    bus.connect().await.expect("connect");

    let ic = bus
        .create_input_context("test-sub-client")
        .await
        .expect("create input context");

    // Subscribe and immediately drop the subscription
    {
        let _sub = ic
            .connect_commit_text(|_text| {
                println!("committed");
            })
            .await
            .expect("subscribe");
        // _sub dropped here — task should be cancelled
    }

    // Should still be able to use the input context
    ic.reset().await.expect("reset after sub drop");

    bus.disconnect().await;
}

#[tokio::test]
async fn test_subscription_manual_cancel() {
    if !has_ibus_daemon() {
        eprintln!("skipping: no ibus-daemon found");
        return;
    }

    let mut bus = Bus::new();
    bus.connect().await.expect("connect");

    let ic = bus
        .create_input_context("test-cancel-client")
        .await
        .expect("create input context");

    let mut sub = ic
        .connect_update_preedit_text(|_text, _cursor, _visible| {})
        .await
        .expect("subscribe");

    // Manually cancel
    sub.cancel();

    // Should still be usable
    ic.reset().await.expect("reset after cancel");

    bus.disconnect().await;
}

// ==================== Factory trait default impl test ====================

#[tokio::test]
async fn test_factory_trait_defaults() {
    struct TestFactory;

    #[async_trait::async_trait]
    impl FactoryImpl for TestFactory {
        async fn create_engine(
            &mut self,
            _engine_name: &str,
        ) -> libibus_rs::Result<Box<dyn EngineImpl>> {
            struct Empty;
            #[async_trait::async_trait]
            impl EngineImpl for Empty {
                async fn process_key_event(
                    &mut self,
                    _event: &KeyEvent,
                    _handle: &libibus_rs::EngineHandle,
                ) -> bool {
                    false
                }
            }
            Ok(Box::new(Empty))
        }
        // destroy_engine uses default
    }

    let mut factory = TestFactory;
    let _engine = factory.create_engine("test").await.unwrap();
    factory.destroy_engine("test").await.unwrap();
}
