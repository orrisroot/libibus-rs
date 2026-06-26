use std::path::Path;
use std::time::Duration;

use libibus_rs::{Component, EngineDesc};

pub const ENGINE_NAME: &str = "libibus-rs-demo";

fn pkg_authors() -> String {
    let authors = env!("CARGO_PKG_AUTHORS");
    authors.split(':').next().unwrap_or(authors).to_owned()
}

fn exec_path_str(path: &Path) -> String {
    path.display().to_string()
}

pub fn build_component(exec_path: &Path) -> Component {
    let mut component = Component::new(
        "com.github.orrisroot.libibus-rs.Demo",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_LICENSE"),
        &pkg_authors(),
        env!("CARGO_PKG_HOMEPAGE"),
        &exec_path_str(exec_path),
    );
    component.set_exec_args(vec!["--ibus"]);

    let mut engine_desc = EngineDesc::new(
        ENGINE_NAME,
        "libibus-rs Demo",
        "Demonstration engine for libibus-rs (Japanese/English input)",
        "ja",
    );
    engine_desc
        .set_license(env!("CARGO_PKG_LICENSE"))
        .set_author(&pkg_authors())
        .set_symbol("あ");

    component.add_engine(engine_desc);
    component
}

pub async fn run_event_loop() {
    loop {
        tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
    }
}

pub fn print_startup_message() {
    println!("{ENGINE_NAME} registered. Waiting for D-Bus requests...");
    println!("Press Ctrl+C to stop.");
}
