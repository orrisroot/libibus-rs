# libibus-rs v1.0.0 API Reference

`libibus-rs` is a pure Rust client library for [IBus (Intelligent Input Bus)](https://github.com/ibus/ibus). It provides D-Bus bindings and high-level wrappers for building IBus engines and interacting with the IBus daemon.

---

## 1. Core Architecture

The crate is structured around four primary components:

* **Bus**: Establishes the D-Bus connection lifecycle with the `ibus-daemon`, registers components, and manages input contexts.
* **EngineImpl**: A user-implementable trait that receives callbacks (key events, focus shifts, panel actions) from the daemon.
* **EngineHandle**: Emits D-Bus signals back to the panel and application (e.g., committing text, updating preedit state, or navigating the lookup table).
* **LookupTable**: An encapsulated data structure representing candidate conversion options displayed in the panel.

---

## 2. API Reference

### Bus
Manages connection establishment, teardown, and component registration.

```rust
pub struct Bus { /* private fields */ }

impl Bus {
    /// Create a disconnected Bus.
    pub fn new() -> Self;

    /// Resolve the IBus address, establish a D-Bus connection, and perform the hello handshake.
    /// It automatically discovers the address from environment variables or `~/.config/ibus/bus/` (supporting order-independent parameter parsing like `unix:guid=...,path=...`).
    pub async fn connect(&mut self) -> Result<()>;

    /// Access the underlying zbus Connection.
    pub fn connection(&self) -> Result<&zbus::Connection>;

    /// Register a component and its engines with the daemon.
    pub async fn register_component(&self, component: &Component) -> Result<()>;

    /// Disconnect from the daemon.
    pub async fn disconnect(&mut self);
}
```

### EngineImpl
Implement this trait to create your custom input engine. All methods receive a borrowed `&EngineHandle` to perform communication back to IBus.

```rust
#[async_trait]
pub trait EngineImpl: Send {
    /// Process a key event. Return true if the event was handled (consumed).
    async fn process_key_event(&mut self, event: &KeyEvent, handle: &EngineHandle) -> bool {
        false
    }

    /// The input context gained focus.
    async fn focus_in(&mut self, handle: &EngineHandle) {}

    /// The input context lost focus.
    async fn focus_out(&mut self, handle: &EngineHandle) {}

    /// Reset the engine state.
    async fn reset(&mut self, handle: &EngineHandle) {}

    /// The engine was enabled by the user.
    async fn enable(&mut self, handle: &EngineHandle) {}

    /// The engine was disabled by the user.
    async fn disable(&mut self, handle: &EngineHandle) {}

    /// Lookup table page navigation requests.
    async fn page_up(&mut self, handle: &EngineHandle) {}
    async fn page_down(&mut self, handle: &EngineHandle) {}
    async fn cursor_up(&mut self, handle: &EngineHandle) {}
    async fn cursor_down(&mut self, handle: &EngineHandle) {}

    /// A candidate in the lookup table was clicked.
    async fn candidate_clicked(&mut self, index: u32, button: u32, state: u32, handle: &EngineHandle) {}

    /// A property (button/menu item) was activated by the user.
    async fn property_activate(&mut self, prop_name: &str, prop_state: u32, handle: &EngineHandle) {}

    /// The engine is being destroyed.
    async fn destroy(&mut self, handle: &EngineHandle) {}
}
```

### EngineHandle
Used to notify the IBus panel or application of state updates.

```rust
#[derive(Clone)]
pub struct EngineHandle { /* private fields */ }

impl EngineHandle {
    /// Emit the CommitText signal to output final characters.
    pub async fn commit_text(&self, text: &str) -> zbus::Result<()>;

    /// Emit the UpdatePreeditText signal to show in‑line composition.
    pub async fn update_preedit_text(&self, text: &str, cursor_pos: u32, visible: bool) -> zbus::Result<()>;

    /// Hide or show the preedit interface.
    pub async fn show_preedit_text(&self) -> zbus::Result<()>;
    pub async fn hide_preedit_text(&self) -> zbus::Result<()>;

    /// Update the candidate panel with a LookupTable.
    pub async fn update_lookup_table(&self, lookup_table: LookupTable, visible: bool) -> zbus::Result<()>;
    pub async fn hide_lookup_table(&self) -> zbus::Result<()>;

    /// Forward a key event back to the application (if not consumed).
    pub async fn forward_key_event(&self, keyval: u32, keycode: u32, state: u32) -> zbus::Result<()>;

    /// Request the deletion of surrounding text around the cursor.
    pub async fn delete_surrounding_text(&self, offset_from_cursor: i32, nchars: u32) -> zbus::Result<()>;
}
```

### LookupTable
Represents conversion candidates. Fields are private to guarantee state safety.

```rust
pub struct LookupTable { /* private fields */ }

impl LookupTable {
    pub fn new() -> Self;

    /// Clear all candidates.
    pub fn clear(&mut self);

    /// Append a conversion option candidate.
    pub fn append_candidate(&mut self, candidate: Text);

    /// Set the absolute cursor index.
    pub fn set_cursor_pos(&mut self, pos: u32);
    pub fn cursor_pos(&self) -> u32;

    /// Set candidates per page.
    pub fn set_page_size(&mut self, size: u32);
    pub fn page_size(&self) -> u32;

    /// Accessors for checking current state
    pub fn candidates(&self) -> &[Text];
    pub fn labels(&self) -> &[Text];
    pub fn cursor_pos_in_page(&self) -> u32;
    pub fn get_current_candidate(&self) -> Option<&Text>;
}
```

---

## 3. Complete Code Example

Below is a complete, minimal example implementing a custom engine:

```rust
use libibus_rs::{Bus, Component, EngineDesc, EngineImpl, EngineHandle, KeyEvent};
use async_trait::async_trait;

struct SimpleEngine;

#[async_trait]
impl SimpleEngine {
    async fn process_key_event(&mut self, event: &KeyEvent, handle: &EngineHandle) -> bool {
        // Echo characters back
        if let Some(c) = char::from_u32(event.keyval) {
            if c.is_ascii_alphanumeric() {
                let _ = handle.commit_text(&c.to_string()).await;
                return true; // Key was consumed
            }
        }
        false // Let the client handle the key
    }
}
```

---

## 4. GVariant Serialization & `IBusSerializable`

Core IBus structures (`Text`, `Attr`, `AttrList`, `LookupTable`, `Prop`, `PropList`) implement the custom serialization layer conforming to GLib's `GVariant` format and the IBus wire format (`IBusSerializable`).

* **Serialization Layout**: Structs are serialized as `(class_name, attachments, ...fields)` flat D-Bus structure tuples.
* **Compatibility**: Works directly with GLib/GDBus-based `ibus-daemon` installations without referencing any external C libraries.
* **Optimization**: Signature parsing is cached using static/atomic initialization to maximize D-Bus message construction throughput.

