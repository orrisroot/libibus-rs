# libibus-rs

A pure Rust client library for [IBus](https://github.com/ibus/ibus) (Intelligent Input Bus).

## Features

- **GVariant & IBus Compatibility** — Full wire-format compatibility with GLib/GDBus-based `ibus-daemon`. Custom `IBusSerializable` serialization layer produces exact `(class_name, attachments, ...fields)` GVariant structures without C library dependencies.
- **Bus** — Auto-discover and connect to `ibus-daemon` via UNIX socket or abstract socket. Handles complex D-Bus addresses with arbitrary parameter order (e.g. `unix:guid=...,path=...`). Supports component registration and input context creation.
- **Engine** — `EngineImpl` trait for implementing input method engines (key events, preedit, candidates, properties, lookup navigation)
- **EngineHandle** — Emit D-Bus signals: commit text, preedit, lookup table, auxiliary text, properties, delete surrounding text, and more
- **Factory** — `FactoryImpl` trait for creating engine instances on demand via `org.freedesktop.IBus.Factory`
- **InputContext** — Client-side input context with full signal subscription (commit, preedit, lookup table, etc.) and cancellable `Subscription` handles
- **Component** — Component/engine metadata with exact D-Bus serialization matching IBus protocol
- **XML** — Generate component XML files for ibus registration
- **Text** — Rich text with underline/foreground/background/font attributes
- **LookupTable** — Candidate word table with page navigation and cursor management
- **Properties** — Engine UI properties (toggle, radio, menu, separator) with sub-property support
- **KeyEvent** — Key event with modifier state and 230+ X11 keysym constants
- **Config** — Configuration get/set via `org.freedesktop.IBus.Config` with cancellable value-change signal subscription
- **Panel** — Panel proxy (`org.freedesktop.IBus.Panel`) for focus notification and engine activation signals
- **Caps / Purpose / Hint** — Input context capability flags and content type constants

## Architecture

```text
┌─────────────────────────────────────────────────────┐
│                  Your Application                   │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │  InputContext (client)                        │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │  process_key_event / focus_in / reset   │  │  │
│  │  │  connect_commit_text / preedit / ...    │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  │                    │ signals                  │  │
│  │                    ▼                          │  │
│  │  Engine (server)                              │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │  EngineImpl trait (user code)           │  │  │
│  │  │  EngineHandle (emit signals)            │  │  │
│  │  │  FactoryImpl (create engines)           │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
│                       │                             │
│            ┌──────────▼────────────┐                │
│            │   Bus                 │                │
│            │   (ibus-daemon conn)  │                │
│            └───────────────────────┘                │
└─────────────────────────────────────────────────────┘
```

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
libibus-rs = { version = "1.0.0" }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

### Minimal engine

```rust
use libibus_rs::{Bus, Component, EngineDesc, EngineImpl, EngineHandle, KeyEvent, FactoryImpl};

struct MyEngine;

#[async_trait::async_trait]
impl EngineImpl for MyEngine {
    async fn process_key_event(&mut self, event: &KeyEvent, handle: &EngineHandle) -> bool {
        // Handle key event, return true if handled
        false
    }

    async fn focus_in(&mut self, handle: &EngineHandle) {
        // Input context gained focus
    }

    async fn focus_out(&mut self, handle: &EngineHandle) {
        // Input context lost focus
    }
}

struct MyFactory;

#[async_trait::async_trait]
impl FactoryImpl for MyFactory {
    async fn create_engine(&mut self, engine_name: &str) -> libibus_rs::Result<Box<dyn EngineImpl>> {
        Ok(Box::new(MyEngine))
    }
}

#[tokio::main]
async fn main() -> libibus_rs::Result<()> {
    let mut bus = Bus::new();
    bus.connect().await?;

    // Create and register component
    let mut component = Component::new(
        "my-engine",
        "My IME",
        "1.0.0",
        "MIT",
        "Author",
        "https://example.com",
        "/usr/lib/ibus/ibus-engine-my-engine",
    );

    let engine_desc = EngineDesc::new(
        "my-engine",
        "My Input Method",
        "A sample input method",
        "ja",
    );
    component.add_engine(engine_desc);

    // Optionally watch for configuration file changes
    component.watch_paths.push("/etc/my-engine.conf".into());

    // Register factory on the bus connection
    let conn = bus.connection()?.clone();
    libibus_rs::factory::register(&conn, Box::new(MyFactory)).await?;

    bus.register_component(&component).await?;

    // Keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
```

### Client-side InputContext

```rust
use libibus_rs::Bus;

#[tokio::main]
async fn main() -> libibus_rs::Result<()> {
    let mut bus = Bus::new();
    bus.connect().await?;

    // Create an input context via the daemon
    let ic = bus.create_input_context("my-app").await?;

    // Subscribe to signals (returns a Subscription that cancels when dropped)
    ic.connect_commit_text(|text| {
        println!("Committed: {}", text);
    }).await?;

    ic.connect_update_preedit_text(|text, cursor_pos, visible, mode| {
        println!("Preedit: {} (cursor={}, visible={}, mode={})", text, cursor_pos, visible, mode);
    }).await?;

    ic.focus_in().await?;
    // ... process key events ...
    ic.focus_out().await?;

    Ok(())
}
```

## Module Overview

| Module | Description |
|--------|-------------|
| `bus` | Connection to `ibus-daemon`, component registration, input context creation |
| `engine` | `EngineImpl` trait + D-Bus `org.freedesktop.IBus.Engine` interface |
| `factory` | `FactoryImpl` trait + D-Bus `org.freedesktop.IBus.Factory` interface |
| `input_context` | Client-side `InputContext` with signal subscriptions and `Subscription` handles |
| `component` | `Component` and `EngineDesc` metadata |
| `text` | `Text` with optional `AttrList` |
| `attr` | Text attributes (underline, foreground, background, etc.) |
| `key` | `KeyEvent`, `ModifierType`, 230+ `keysym` constants |
| `lookup_table` | `LookupTable` with page navigation |
| `prop` | `Prop` and `PropList` for engine UI |
| `config` | `Config` proxy for `org.freedesktop.IBus.Config` |
| `panel` | `Panel` proxy for `org.freedesktop.IBus.Panel` |
| `address` | IBus socket address resolution from `~/.config/ibus/bus/` |
| `xml` | Component XML file generation |
| `dbus` | Low-level D-Bus proxy/interface definitions |
| `error` | Error types |
| `signal` | Signal handler task with panic resilience |
| `serializable` | `IBusSerializable` trait for GVariant serialization |

## License

MIT
