use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use crate::component::{Component, EngineDesc};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Serialize an IBus component definition to XML.
///
/// The output conforms to the IBus component XML format used by
/// `ibus-daemon` for discovering and registering input method engines.
///
/// # Example
///
/// ```rust
/// use libibus_rs::{Component, EngineDesc};
///
/// let mut component = Component::new(
///     "com.example.Test", "Test", "1.0", "MIT",
///     "Author", "https://example.com", "/usr/libexec/ibus-engine-test",
/// );
/// component.add_engine(EngineDesc::new("test", "Test Engine", "A test", "en"));
///
/// let xml = libibus_rs::xml::component_to_xml(&component);
/// assert!(xml.starts_with("<?xml"));
/// ```
pub fn component_to_xml(component: &Component) -> String {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    write_event(
        &mut writer,
        Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)),
    );
    let comment = format!(" filename: {}.xml ", component.name);
    write_event(&mut writer, Event::Comment(BytesText::new(&comment)));
    write_event(&mut writer, Event::Start(BytesStart::new("component")));
    element(&mut writer, "name", &component.name);
    element(&mut writer, "description", &component.description);
    element(&mut writer, "version", &component.version);
    element(&mut writer, "license", &component.license);
    element(&mut writer, "author", &component.author);
    element(&mut writer, "homepage", &component.homepage);
    if !component.text_domain.is_empty() {
        element(&mut writer, "textdomain", &component.text_domain);
    }

    write_exec(&mut writer, &component.exec_path, &component.exec_args);

    for watch_path in &component.watch_paths {
        write_event(&mut writer, Event::Start(BytesStart::new("watch")));
        write_event(&mut writer, Event::Text(BytesText::new(watch_path)));
        write_event(&mut writer, Event::End(BytesEnd::new("watch")));
    }

    for engine in &component.engines {
        engine_to_xml(&mut writer, engine);
    }

    write_event(&mut writer, Event::End(BytesEnd::new("component")));

    String::from_utf8(writer.into_inner()).expect("quick-xml output should be valid UTF-8")
}

fn engine_to_xml(writer: &mut Writer<Vec<u8>>, engine: &EngineDesc) {
    write_event(writer, Event::Start(BytesStart::new("engine")));
    element(writer, "name", &engine.name);
    element(writer, "longname", &engine.longname);
    element(writer, "description", &engine.description);
    element(writer, "language", &engine.language);
    if !engine.license.is_empty() {
        element(writer, "license", &engine.license);
    }
    if !engine.author.is_empty() {
        element(writer, "author", &engine.author);
    }
    if !engine.icon.is_empty() {
        element(writer, "icon", &engine.icon);
    }
    if !engine.layout.is_empty() && engine.layout != "us" {
        element(writer, "layout", &engine.layout);
    }
    if !engine.hotkeys.is_empty() {
        element(writer, "hotkeys", &engine.hotkeys.join(","));
    }
    element(writer, "rank", &engine.rank.to_string());
    if !engine.symbol.is_empty() {
        element(writer, "symbol", &engine.symbol);
    }
    if !engine.setup.is_empty() {
        element(writer, "setup", &engine.setup);
    }
    if !engine.layout_variants.is_empty() {
        element(writer, "layout_variants", &engine.layout_variants);
    }
    if !engine.layout_option.is_empty() {
        element(writer, "layout_option", &engine.layout_option);
    }
    element(writer, "version", &engine.version);
    if !engine.text_domain.is_empty() {
        element(writer, "textdomain", &engine.text_domain);
    }
    write_event(writer, Event::End(BytesEnd::new("engine")));
}

fn element(writer: &mut Writer<Vec<u8>>, tag: &str, value: &str) {
    let escaped = escape_xml(value);
    write_event(writer, Event::Start(BytesStart::new(tag)));
    write_event(writer, Event::Text(BytesText::new(&escaped)));
    write_event(writer, Event::End(BytesEnd::new(tag)));
}

fn write_exec(writer: &mut Writer<Vec<u8>>, exec_path: &str, exec_args: &[String]) {
    let mut elem = BytesStart::new("exec");
    if !exec_path.is_empty() {
        elem.push_attribute(("exec", escape_xml(exec_path).as_str()));
    }
    if !exec_args.is_empty() {
        let args = exec_args
            .iter()
            .map(|a| escape_xml(a))
            .collect::<Vec<_>>()
            .join(" ");
        elem.push_attribute(("args", args.as_str()));
    }
    write_event(writer, Event::Empty(elem));
}

fn write_event(writer: &mut Writer<Vec<u8>>, event: Event<'_>) {
    writer
        .write_event(event)
        .expect("quick-xml write to Vec<u8> should never fail");
}
