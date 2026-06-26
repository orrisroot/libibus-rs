#![allow(non_upper_case_globals)]
#![doc = "A pure Rust client library for [IBus](https://github.com/ibus/ibus) (Intelligent Input Bus).\n\nThis crate provides D-Bus bindings and high-level wrappers for building IBus engines\nand interacting with the IBus daemon.\n\n# Architecture\n\n- **`Bus`** — connect to the ibus-daemon, register components\n- **`EngineImpl`** / **`FactoryImpl`** — implement these traits to create custom input engines\n- **`EngineHandle`** — emit signals (commit text, show preedit, update lookup table, etc.)\n- **`Config`** / **`Panel`** — interact with the IBus configuration daemon and panel\n- **`KeyEvent`** / **`ModifierType`** — handle keyboard input\n- **`Text`**, **`AttrList`**, **`LookupTable`**, **`PropList`** — rich text, attributes, candidates, and property models\n\n# Quick start\n\n```rust,no_run\nuse libibus_rs::{Bus, EngineDesc, Component};\n\n#[tokio::main]\nasync fn main() -> libibus_rs::Result<()> {\n    let mut bus = Bus::new();\n    bus.connect().await?;\n\n    let engine = EngineDesc::new(\"my-engine\", \"My Engine\", \"An IBus engine\", \"ja\");\n    let mut component = Component::new(\n        \"com.example.MyEngine\", \"My IME\", \"1.0\", \"MIT\",\n        \"Author\", \"https://example.com\", \"/usr/libexec/ibus-engine-my\",\n    );\n    bus.register_component(component.add_engine(engine)).await?;\n\n    // See `examples/demo_engine.rs` for a complete example.\n    Ok(())\n}\n```"]

pub mod attr;
pub mod bus;
pub mod component;
pub mod config;
pub mod engine;
pub mod error;
pub mod factory;
pub mod input_context;
pub mod key;
pub mod lookup_table;
pub mod panel;
pub mod prop;
pub mod text;
pub mod xml;

// Internal modules
pub(crate) mod address;
pub(crate) mod conn;
pub(crate) mod dbus;
pub mod serializable;
pub(crate) mod signal;

pub use serializable::IBusSerializable;

// Re-exports for convenience
pub use attr::{Attr, AttrList, AttrType};
pub use bus::Bus;
pub use component::{Component, EngineDesc};
pub use config::Config;
pub use engine::{EngineHandle, EngineImpl};
pub use error::{Error, Result};
pub use factory::FactoryImpl;
pub use input_context::Hint as InputHint;
pub use input_context::{InputContext, Purpose, Subscription};
pub use key::keysym;
pub use key::{KeyEvent, ModifierType};
pub use lookup_table::LookupTable;
pub use panel::Panel;
pub use prop::{Prop, PropList, PropState, PropType};
pub use text::Text;
pub use xml::component_to_xml;

/// Connect to the D-Bus session bus.
///
/// Use this connection to register your engine factory so the ibus-daemon can
/// discover it via `NameOwnerChanged` on the session bus.
///
/// # Errors
///
/// Returns [`Error::Connection`] if the session bus cannot be reached.
pub async fn connect_session() -> Result<zbus::Connection> {
    conn::connect_session().await
}
