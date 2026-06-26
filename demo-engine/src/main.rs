mod engine;
mod framework;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;

use libibus_rs::Bus;
use libibus_rs::error::Result;

use crate::framework::ENGINE_NAME;

fn log_msg(msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/demo-engine.log")
    {
        let _ = writeln!(f, "[{}] {}", std::process::id(), msg);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let exec_path =
        env::current_exe().unwrap_or_else(|_| "/usr/libexec/libibus-rs-engine-demo".into());

    if args.len() > 1 && args[1] == "--xml" {
        let component = framework::build_component(&exec_path);
        println!("{}", libibus_rs::component_to_xml(&component));
        return Ok(());
    }

    let launched_by_daemon = args.iter().any(|a| a == "--ibus");
    log_msg(&format!("started (daemon={})", launched_by_daemon));

    // Connect to the ibus private bus.
    // The C library uses a single connection to the ibus private bus for all
    // operations: factory registration and name request.
    let mut bus = Bus::new();
    log_msg("connecting to ibus-daemon (private bus)...");
    if let Err(e) = bus.connect().await {
        log_msg(&format!("private bus connect failed: {}", e));
        return Err(e);
    }
    log_msg("connected to private bus");

    let private_conn = bus.connection()?;
    let component = framework::build_component(&exec_path);

    // Register the factory on the private bus connection.
    // The daemon's bus_factory_proxy_new() creates a GDBusProxy on the
    // engine's private bus connection. The proxy defaults to the connection's
    // unique bus name as the target, so our factory must be on this connection.
    log_msg("registering factory on private bus...");
    match libibus_rs::factory::register(private_conn, Box::new(engine::DemoFactory)).await {
        Ok(()) => log_msg("factory registered on private bus"),
        Err(e) => log_msg(&format!("factory register failed: {}", e)),
    }

    // Request the component name on the private bus.
    // The daemon watches for NameOwnerChanged on the private bus.
    // When our name appears, it creates a factory proxy pointing to us and
    // then calls CreateEngine. This replaces the need for RegisterComponent.
    log_msg(&format!(
        "requesting name '{}' on private bus...",
        component.name
    ));
    match private_conn.request_name(component.name.clone()).await {
        Ok(_) => log_msg("name requested on private bus"),
        Err(e) => log_msg(&format!("name request failed: {}", e)),
    }

    if launched_by_daemon {
        println!("{ENGINE_NAME} ready.");
    } else {
        framework::print_startup_message();
    }
    framework::run_event_loop().await;
    Ok(())
}
