# libibus-rs API Reference

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
    /// It automatically discovers the address from environment variables or `~/.config/ibus/bus/`
    /// (supporting order-independent parameter parsing like `unix:guid=...,path=...`).
    pub async fn connect(&mut self) -> Result<()>;

    /// Check whether the bus is currently connected.
    pub fn is_connected(&self) -> bool;

    /// Access the underlying zbus Connection.
    pub fn connection(&self) -> Result<&zbus::Connection>;

    /// Register a component and its engines with the daemon.
    pub async fn register_component(&self, component: &Component) -> Result<()>;

    /// Get the daemon's address string.
    pub async fn get_address(&self) -> Result<String>;

    /// Switch the global input method engine.
    pub async fn set_global_engine(&self, engine_name: &str) -> Result<()>;

    /// Create a new InputContext via the ibus-daemon.
    pub async fn create_input_context(&self, client_name: &str) -> Result<InputContext>;

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

    /// The cursor location changed.
    async fn set_cursor_location(&mut self, x: i32, y: i32, w: i32, h: i32, handle: &EngineHandle) {}

    /// The content type of the focused input field changed.
    async fn set_content_type(&mut self, purpose: u32, hints: u32, handle: &EngineHandle) {}

    /// Surrounding text around the cursor was provided.
    async fn set_surrounding_text(&mut self, text: &str, cursor_pos: u32, anchor_pos: u32, handle: &EngineHandle) {}

    /// The engine name was set by the daemon.
    async fn set_engine_name(&mut self, name: &str, handle: &EngineHandle) {}

    /// Lookup table page navigation requests.
    async fn page_up(&mut self, handle: &EngineHandle) {}
    async fn page_down(&mut self, handle: &EngineHandle) {}
    async fn cursor_up(&mut self, handle: &EngineHandle) {}
    async fn cursor_down(&mut self, handle: &EngineHandle) {}

    /// A candidate in the lookup table was clicked.
    async fn candidate_clicked(&mut self, index: u32, button: u32, state: u32, handle: &EngineHandle) {}

    /// A property (button/menu item) was activated by the user.
    async fn property_activate(&mut self, prop_name: &str, prop_state: u32, handle: &EngineHandle) {}

    /// Show a specific property in the panel.
    async fn property_show(&mut self, prop_name: &str, handle: &EngineHandle) {}

    /// Hide a specific property in the panel.
    async fn property_hide(&mut self, prop_name: &str, handle: &EngineHandle) {}

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
    pub async fn commit_text(&self, text: impl Into<crate::text::Text>) -> zbus::Result<()>;

    /// Emit the UpdatePreeditText signal to show in‑line composition.
    /// `mode` controls preedit behavior on focus out (0 = clear, 1 = commit).
    pub async fn update_preedit_text(&self, text: impl Into<crate::text::Text>, cursor_pos: u32, visible: bool, mode: u32) -> zbus::Result<()>;

    /// Hide or show the preedit interface.
    pub async fn show_preedit_text(&self) -> zbus::Result<()>;
    pub async fn hide_preedit_text(&self) -> zbus::Result<()>;

    /// Update the candidate panel with a LookupTable.
    pub async fn update_lookup_table(&self, lookup_table: LookupTable, visible: bool) -> zbus::Result<()>;
    pub async fn show_lookup_table(&self) -> zbus::Result<()>;
    pub async fn hide_lookup_table(&self) -> zbus::Result<()>;

    /// Update the auxiliary text displayed by the panel.
    pub async fn update_auxiliary_text(&self, text: impl Into<crate::text::Text>, visible: bool) -> zbus::Result<()>;
    pub async fn show_auxiliary_text(&self) -> zbus::Result<()>;
    pub async fn hide_auxiliary_text(&self) -> zbus::Result<()>;

    /// Register properties for the engine UI.
    pub async fn register_properties(&self, props: PropList) -> zbus::Result<()>;

    /// Update a specific property.
    pub async fn update_property(&self, prop: Prop) -> zbus::Result<()>;

    /// Forward a key event back to the application (if not consumed).
    pub async fn forward_key_event(&self, keyval: u32, keycode: u32, state: u32) -> zbus::Result<()>;

    /// Request the deletion of surrounding text around the cursor.
    pub async fn delete_surrounding_text(&self, offset_from_cursor: i32, nchars: u32) -> zbus::Result<()>;

    /// Lookup table navigation signals.
    pub async fn page_up_lookup_table(&self) -> zbus::Result<()>;
    pub async fn page_down_lookup_table(&self) -> zbus::Result<()>;
    pub async fn cursor_up_lookup_table(&self) -> zbus::Result<()>;
    pub async fn cursor_down_lookup_table(&self) -> zbus::Result<()>;

    /// Request the client to send surrounding text via SetSurroundingText.
    pub async fn require_surrounding_text(&self) -> zbus::Result<()>;
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

    /// Replace all candidates.
    pub fn set_candidates(&mut self, candidates: Vec<Text>);

    /// Set the absolute cursor index.
    pub fn set_cursor_pos(&mut self, pos: u32);
    pub fn cursor_pos(&self) -> u32;

    /// Set candidates per page.
    pub fn set_page_size(&mut self, size: u32);
    pub fn page_size(&self) -> u32;

    /// Set the panel layout orientation.
    pub fn set_orientation(&mut self, orientation: LookupOrientation);
    pub fn orientation(&self) -> LookupOrientation;

    /// Enable or disable wrapping at list boundaries.
    pub fn set_round(&mut self, round: bool);
    pub fn round(&self) -> bool;

    /// Show or hide the cursor highlight.
    pub fn set_cursor_visible(&mut self, visible: bool);
    pub fn cursor_visible(&self) -> bool;

    /// Replace all candidate labels.
    pub fn set_labels(&mut self, labels: Vec<Text>);

    /// Accessors for checking current state
    pub fn candidates(&self) -> &[Text];
    pub fn labels(&self) -> &[Text];
    pub fn cursor_pos_in_page(&self) -> u32;
    pub fn get_current_candidate(&self) -> Option<&Text>;

    /// Move the cursor to the previous page.
    pub fn page_up(&mut self) -> bool;

    /// Move the cursor to the next page.
    pub fn page_down(&mut self) -> bool;

    /// Move the cursor up by one candidate.
    pub fn cursor_up(&mut self) -> bool;

    /// Move the cursor down by one candidate.
    pub fn cursor_down(&mut self) -> bool;

    /// Return a slice of candidates on the current page.
    pub fn current_page(&self) -> &[Text];

    /// Return the total number of candidates.
    pub fn number_of_candidates(&self) -> u32;

    /// Whether there are no candidates.
    pub fn is_empty(&self) -> bool;
}
```

#### LookupOrientation

Panel layout orientation for the lookup table.

```rust
pub enum LookupOrientation {
    Horizontal = 0,
    Vertical = 1,
    System = 2,
}
```

### Component

Describes an input method component with its engines and metadata.

```rust
pub struct Component {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub author: String,
    pub homepage: String,
    pub text_domain: String,
    pub exec_path: String,
    pub exec_args: Vec<String>,
    pub engines: Vec<EngineDesc>,
    pub watch_paths: Vec<String>,
}

impl Component {
    pub fn new(
        name: &str,
        description: &str,
        version: &str,
        license: &str,
        author: &str,
        homepage: &str,
        exec_path: &str,
    ) -> Self;

    pub fn add_engine(&mut self, engine: EngineDesc) -> &mut Self;
    pub fn set_exec_args(&mut self, args: Vec<&str>) -> &mut Self;
    pub fn set_text_domain(&mut self, text_domain: &str) -> &mut Self;
    pub fn add_watch_path(&mut self, path: &str) -> &mut Self;
}
```

### EngineDesc

Description of a single input method engine within a component.

```rust
pub struct EngineDesc {
    pub name: String,
    pub longname: String,
    pub description: String,
    pub language: String,
    pub license: String,
    pub author: String,
    pub icon: String,
    pub layout: String,
    pub hotkeys: Vec<String>,
    pub rank: u32,
    pub symbol: String,
    pub setup: String,
    pub layout_variants: String,
    pub layout_option: String,
    pub version: String,
    pub text_domain: String,
    pub icon_prop_key: String,
}

impl EngineDesc {
    pub fn new(name: &str, longname: &str, description: &str, language: &str) -> Self;
    pub fn set_license(&mut self, license: &str) -> &mut Self;
    pub fn set_author(&mut self, author: &str) -> &mut Self;
    pub fn set_icon(&mut self, icon: &str) -> &mut Self;
    pub fn set_layout(&mut self, layout: &str) -> &mut Self;
    pub fn set_hotkeys(&mut self, hotkeys: Vec<&str>) -> &mut Self;
    pub fn set_rank(&mut self, rank: u32) -> &mut Self;
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self;
    pub fn set_setup(&mut self, setup: &str) -> &mut Self;
    pub fn set_version(&mut self, version: &str) -> &mut Self;
}
```

### InputContext

Client-side input context for receiving signals from an active input engine.

```rust
pub struct InputContext { /* private fields */ }

impl InputContext {
    /// Create a new InputContext bound to a specific D-Bus object path.
    pub async fn with_path(connection: &Connection, path: &str) -> Result<Self>;

    /// Send a key event to the engine for processing.
    pub async fn process_key_event(&self, keyval: u32, keycode: u32, state: u32) -> Result<bool>;

    /// Notify the engine that this input context gained focus.
    pub async fn focus_in(&self) -> Result<()>;

    /// Notify the engine that this input context lost focus.
    pub async fn focus_out(&self) -> Result<()>;

    /// Reset the engine state.
    pub async fn reset(&self) -> Result<()>;

    /// Switch to a specific input method engine by name.
    pub async fn set_engine(&self, engine_name: &str) -> Result<()>;

    /// Get the name of the current engine.
    pub async fn get_engine(&self) -> Result<String>;

    /// Report the cursor location and size to the engine.
    pub async fn set_cursor_location(&self, x: i32, y: i32, w: i32, h: i32) -> Result<()>;

    /// Set the capabilities supported by this input context.
    pub async fn set_capabilities(&self, caps: Caps) -> Result<()>;

    /// Provide surrounding text around the cursor to the engine.
    pub async fn set_surrounding_text(&self, text: &str, cursor_pos: u32, anchor_pos: u32) -> Result<()>;

    /// Set the content type (purpose + hints) of the input field.
    pub async fn set_content_type(&self, purpose: u32, hints: u32) -> Result<()>;

    /// Subscribe to the `commit-text` signal.
    pub async fn connect_commit_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(String) + Send + 'static;

    /// Subscribe to the `update-preedit-text` signal (mode: 0=clear on focus out, 1=commit).
    pub async fn connect_update_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(String, u32, bool, u32) + Send + 'static;

    /// Subscribe to the `show-preedit-text` signal.
    pub async fn connect_show_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `hide-preedit-text` signal.
    pub async fn connect_hide_preedit_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `update-auxiliary-text` signal.
    pub async fn connect_update_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(String, bool) + Send + 'static;

    /// Subscribe to the `show-auxiliary-text` signal.
    pub async fn connect_show_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `hide-auxiliary-text` signal.
    pub async fn connect_hide_auxiliary_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `update-lookup-table` signal.
    pub async fn connect_update_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(LookupTable, bool) + Send + 'static;

    /// Subscribe to the `show-lookup-table` signal.
    pub async fn connect_show_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `hide-lookup-table` signal.
    pub async fn connect_hide_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `page-up-lookup-table` signal.
    pub async fn connect_page_up_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `page-down-lookup-table` signal.
    pub async fn connect_page_down_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `cursor-up-lookup-table` signal.
    pub async fn connect_cursor_up_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `cursor-down-lookup-table` signal.
    pub async fn connect_cursor_down_lookup_table<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `forward-key-event` signal.
    pub async fn connect_forward_key_event<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(u32, u32, u32) + Send + 'static;

    /// Subscribe to the `delete-surrounding-text` signal.
    pub async fn connect_delete_surrounding_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(i32, u32) + Send + 'static;

    /// Subscribe to the `disabled` signal.
    pub async fn connect_disabled<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;

    /// Subscribe to the `require-surrounding-text` signal.
    pub async fn connect_require_surrounding_text<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn() + Send + 'static;
}
```

### Subscription

A handle returned by connection subscription methods. When dropped, the associated signal handler task is cancelled.

```rust
#[must_use = "dropping this handle cancels the signal subscription"]
pub struct Subscription { /* private fields */ }

impl Subscription {
    /// Manually cancel the signal handler task.
    pub fn cancel(&mut self);
}
```

### Config

Client for the IBus configuration daemon (`org.freedesktop.IBus.Config`).

```rust
pub struct Config { /* private fields */ }

impl Config {
    pub async fn new(connection: &Connection) -> Result<Self>;
    pub async fn get_value(&self, section: &str, name: &str) -> Result<OwnedValue>;
    pub async fn get_string(&self, section: &str, name: &str) -> Result<String>;
    pub async fn get_i32(&self, section: &str, name: &str) -> Result<i32>;
    pub async fn get_bool(&self, section: &str, name: &str) -> Result<bool>;
    pub async fn get_f64(&self, section: &str, name: &str) -> Result<f64>;
    pub async fn set_value(&self, section: &str, name: &str, value: &zvariant::Value<'_>) -> Result<()>;
    pub async fn unset(&self, section: &str, name: &str) -> Result<()>;
    pub async fn get_values(&self, section: &str) -> Result<Vec<(String, OwnedValue)>>;
    pub async fn connect_value_changed<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(String, String, OwnedValue) + Send + 'static;
}
```

### Panel

Client for the IBus panel (`org.freedesktop.IBus.Panel`).

```rust
pub struct Panel { /* private fields */ }

impl Panel {
    pub async fn new(connection: &Connection) -> Result<Self>;
    pub async fn focus_in(&self) -> Result<()>;
    pub async fn focus_out(&self) -> Result<()>;
    pub async fn reset(&self) -> Result<()>;
    pub async fn connect_engine_activate<F>(&self, callback: F) -> Result<Subscription>
    where F: Fn(String) + Send + 'static;
}
```

### Text

A string with optional text attributes.

```rust
pub struct Text {
    pub text: String,
    pub attrs: AttrList,
    pub cursor_pos: u32,
}

impl Text {
    pub fn new(text: &str) -> Self;
    pub fn with_attrs(text: &str, attrs: AttrList) -> Self;
    pub fn with_cursor(text: &str, attrs: AttrList, cursor_pos: u32) -> Self;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn append(&mut self, other: &str);
    pub fn clear(&mut self);
}
```

### Attr / AttrList

Text attributes for formatting (underline, foreground colour, background colour, etc.).

```rust
pub enum AttrType {
    Underline = 1,
    Foreground = 2,
    Background = 3,
    FontStyle = 4,
    FontWeight = 5,
    Rise = 6,
    Strikethrough = 7,
    Scale = 8,
    Align = 9,
}

pub struct Attr {
    pub attr_type: u32,
    pub value: u32,
    pub start_index: u32,
    pub end_index: u32,
}

impl Attr {
    pub fn new(attr_type: AttrType, value: u32, start_index: u32, end_index: u32) -> Self;
    pub fn underline(style: u32, start_index: u32, end_index: u32) -> Self;
    pub fn foreground(color: u32, start_index: u32, end_index: u32) -> Self;
    pub fn background(color: u32, start_index: u32, end_index: u32) -> Self;
}

pub struct AttrList {
    pub attrs: Vec<Attr>,
}

impl AttrList {
    pub fn new() -> Self;
    pub fn append(&mut self, attr: Attr);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### Prop / PropList

Engine UI properties (toggle, radio, menu, separator).

```rust
pub enum PropType {
    Normal = 0,
    Toggle = 1,
    Radio = 2,
    Menu = 3,
    Separator = 4,
}

pub enum PropState {
    Unchecked = 0,
    Checked = 1,
    Inconsistent = 2,
}

pub struct Prop {
    pub key: String,
    pub prop_type: u32,
    pub label: Text,
    pub icon: String,
    pub tooltip: Text,
    pub sensitive: bool,
    pub visible: bool,
    pub state: u32,
    pub sub_props: Box<PropList>,
    pub symbol: Text,
}

impl Prop {
    pub fn new(key: &str, label: &str) -> Self;
    pub fn separator() -> Self;
    pub fn toggle(key: &str, label: &str) -> Self;
    pub fn radio(key: &str, label: &str) -> Self;
    pub fn set_label(&mut self, label: &str) -> &mut Self;
    pub fn set_icon(&mut self, icon: &str) -> &mut Self;
    pub fn set_tooltip(&mut self, tooltip: &str) -> &mut Self;
    pub fn set_sensitive(&mut self, sensitive: bool) -> &mut Self;
    pub fn set_visible(&mut self, visible: bool) -> &mut Self;
    pub fn set_state(&mut self, state: PropState) -> &mut Self;
    pub fn set_checked(&mut self, checked: bool) -> &mut Self;
    pub fn set_sub_props(&mut self, sub_props: PropList) -> &mut Self;
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self;
    pub fn is_toggle(&self) -> bool;
    pub fn is_separator(&self) -> bool;
    pub fn is_checked(&self) -> bool;
    pub fn has_sub_props(&self) -> bool;
}

pub struct PropList {
    pub props: Vec<Prop>,
}

impl PropList {
    pub fn new() -> Self;
    pub fn append(&mut self, prop: Prop);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn update_property(&mut self, prop: &Prop);
    pub fn get(&self, key: &str) -> Option<&Prop>;
}
```

### KeyEvent / ModifierType

Keyboard input representation with modifier flags.

```rust
pub struct KeyEvent {
    pub keyval: u32,
    pub keycode: u32,
    pub state: u32,
}

impl KeyEvent {
    pub const fn new(keyval: u32, keycode: u32, state: u32) -> Self;
    pub fn modifiers(&self) -> ModifierType;
    pub fn set_modifiers(&mut self, modifiers: ModifierType);
    pub fn is_modifier(&self) -> bool;
}

bitflags! {
    pub struct ModifierType: u32 {
        const SHIFT = 1 << 0;
        const LOCK = 1 << 1;
        const CONTROL = 1 << 2;
        const MOD1 = 1 << 3;
        const MOD2 = 1 << 4;
        const MOD3 = 1 << 5;
        const MOD4 = 1 << 6;
        const MOD5 = 1 << 7;
        const BUTTON1 = 1 << 8;
        const BUTTON2 = 1 << 9;
        const BUTTON3 = 1 << 10;
        const BUTTON4 = 1 << 11;
        const BUTTON5 = 1 << 12;
        const HANDLED = 1 << 24;
        const FORWARD = 1 << 25;
        const RELEASE = 1 << 30;
    }
}

impl ModifierType {
    pub fn is_shift(self) -> bool;
    pub fn is_control(self) -> bool;
    pub fn is_alt(self) -> bool;
    pub fn is_super(self) -> bool;
    pub fn is_release(self) -> bool;
    pub fn is_handled(self) -> bool;
}
```

### Caps / Purpose / Hint

Input context capability flags and content type constants.

```rust
bitflags! {
    pub struct Caps: u32 {
        const SURROUNDING = 1 << 0;
        const SURROUNDING_SELECTION = 1 << 1;
        const PREEDIT_ATTR = 1 << 2;
        const AUX = 1 << 3;
        const LOOKUP_TABLE = 1 << 4;
        const CONTENT_HINT = 1 << 5;
        const CONTENT_PURPOSE = 1 << 6;
        const DELETE_SURROUNDING = 1 << 7;
    }
}

pub enum Purpose {
    Normal = 0,
    Alpha = 1,
    Name = 2,
    Number = 3,
    Pin = 4,
    Email = 5,
    Url = 6,
    Password = 7,
    Caption = 8,
}

bitflags! {
    pub struct Hint: u32 {
        const NONE = 0;
        const NO_AUTO_CAPS = 1 << 0;
        const NO_AUTO_CORRECTION = 1 << 1;
        const NO_PREDICTIVE = 1 << 2;
        const LOWERCASE = 1 << 3;
        const UPPERCASE = 1 << 4;
        const TITLECASE = 1 << 5;
        const HIDDEN_TEXT = 1 << 6;
        const SENSITIVE_DATA = 1 << 7;
        const COMPLETION = 1 << 8;
    }
}
```

### FactoryImpl

User-implementable trait for creating engine instances.

```rust
#[async_trait]
pub trait FactoryImpl: Send {
    async fn create_engine(&mut self, engine_name: &str) -> Result<Box<dyn EngineImpl>>;
    async fn destroy_engine(&mut self, engine_name: &str) -> Result<()> { Ok(()) }
}

pub async fn register(conn: &Connection, impl_: Box<dyn FactoryImpl>) -> Result<()>;
```

---

## 3. GVariant Serialization & `IBusSerializable`

Core IBus structures (`Component`, `EngineDesc`, `Text`, `Attr`, `AttrList`, `LookupTable`, `Prop`, `PropList`) implement the custom serialization layer conforming to GLib's `GVariant` format and the IBus wire format (`IBusSerializable`).

* **Serialization Layout**: Structs are serialized as `(class_name, attachments, ...fields)` flat D-Bus structure tuples. For example, `Component` uses the exact signature `(sa{sv}ssssssssavav)`, carefully maintaining field order (`observed_paths` before `engines`), dynamically constructing internal structs (`IBusObservedPath`, `IBusEngineDesc`), and wrapping them in array variants (`av`) as strictly expected by the C implementation.
* **EngineDesc**: Hotkeys are serialized as a space-separated string. Includes `icon_prop_key` field for dynamic panel icons. Rank is serialized as `int32` per the IBus protocol.
* **ObservedPath**: Serialized as `(path, mtime)` — matching the Python/C reference exactly (no extra fields).
* **PropList**: Serialized as an array of IBusProperty structures (variant-wrapped). Uses a manual `Type` implementation to break the recursive `Prop ↔ Box<PropList>` cycle.
* **Compatibility**: Works directly with GLib/GDBus-based `ibus-daemon` installations without referencing any external C libraries. The serialization logic has been verified for full structural compatibility with the IBus D-Bus wire format.
* **Optimization**: Signature parsing is cached using static/atomic initialization to maximize D-Bus message construction throughput.

## 4. Concurrency Model & Signal Resilience

* **zbus 5.x with Tokio**: The library uses `zbus` 5.x with the `tokio` feature for tight async runtime integration. No extra threads are spawned — all D-Bus I/O runs on the tokio reactor.
* **D-Bus Proxy Methods**: All D-Bus proxy trait methods (`IBus`, `FactoryProxy`, `Panel`, `Config`, `InputContext`, `Engine`) use `&self` instead of `&mut self`.
* **EngineHandle**: Signal emission uses `SignalEmitter::clone()` for thread-safe signal dispatch.
* **Panic Resilience**: Signal handler tasks wrap handler closures in `catch_unwind`. Panics are caught, logged, and the signal processing pipeline continues running.
* **Mutex Design**: Engine and factory implementations use `tokio::sync::Mutex` with the lock released before D-Bus operations to prevent deadlocks.

## 5. Error Types

```rust
pub enum Error {
    DBus(zbus::Error),
    Component(String),
    Engine(String),
    Connection(String),
    Address(String),
    NotConnected,
    EngineNotFound(String),
    InvalidArgument(String),
    Io(std::io::Error),
}
```

## 6. Keysym Constants

The `keysym` module re-exports 230+ X11 keysym constants following the `X11/keysymdef.h` naming convention. Key categories include:

* **Editing**: `BackSpace`, `Return`, `Tab`, `Escape`, `Delete`, `Insert`
* **Navigation**: `Home`, `End`, `Prior`/`Page_Up`, `Next`/`Page_Down`, `Up`, `Down`, `Left`, `Right`
* **Function keys**: `F1`–`F35`
* **Modifiers**: `Shift_L`, `Shift_R`, `Control_L`, `Control_R`, `Caps_Lock`, `Alt_L`, `Alt_R`, `Super_L`, `Super_R`, `Meta_L`, `Meta_R`, `Hyper_L`, `Hyper_R`
* **Japanese input**: `Kanji`, `Muhenkan`, `Henkan`, `Hiragana`, `Katakana`, `ZenKaku`, `Hankaku`, `ZenKaku_Hankaku`, `Touroku`, `Massyo`, `SingleCandidate`, `MultipleCandidate`, `PreviousCandidate`
* **Keypad**: `KP_0`–`KP_9`, `KP_Enter`, `KP_Space`, `KP_Tab`, `KP_Add`, `KP_Subtract`, `KP_Multiply`, `KP_Divide`, `KP_Decimal`, `KP_Equal`
* **Alphanumeric**: `a`–`z`, `A`–`Z`, `_0`–`_9`, `space`, `exclam`, `quotedbl`, `at`, `ampersand`, etc.
