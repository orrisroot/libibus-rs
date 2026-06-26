use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, interface};
use zvariant::ObjectPath;
use zvariant::OwnedObjectPath;

use crate::engine::{Engine, EngineHandle, EngineImpl};
use crate::error::{Error, Result};

fn factory_log(msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/demo-engine.log")
    {
        let _ = writeln!(f, "[{}] [factory] {}", std::process::id(), msg);
    }
}

const FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";

static ENGINE_COUNTER: AtomicU32 = AtomicU32::new(0);

fn next_engine_path() -> String {
    let id = ENGINE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("/org/freedesktop/IBus/Engine/{}", id)
}

/// User-implementable trait for creating engine instances.
///
/// The IBus daemon calls [`create_engine`](FactoryImpl::create_engine) when it
/// wants to activate an engine by name. Implement this trait and register it
/// with [`register`] to receive those requests.
///
/// # Example
///
/// ```rust,no_run
/// use libibus_rs::engine::EngineImpl;
/// use libibus_rs::factory::{FactoryImpl, register};
/// use libibus_rs::error::Result;
///
/// struct MyFactory;
///
/// #[async_trait::async_trait]
/// impl FactoryImpl for MyFactory {
///     async fn create_engine(&mut self, engine_name: &str) -> Result<Box<dyn EngineImpl>> {
///         // Create and return your engine implementation
///         todo!()
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait FactoryImpl: Send {
    /// Create a new engine instance for `engine_name`.
    async fn create_engine(&mut self, engine_name: &str) -> Result<Box<dyn EngineImpl>>;

    /// Destroy a previously created engine.
    async fn destroy_engine(&mut self, _engine_name: &str) -> Result<()> {
        Ok(())
    }
}

pub struct Factory {
    inner: Arc<Mutex<Box<dyn FactoryImpl>>>,
    conn: Connection,
}

impl Factory {
    pub fn new(inner: Box<dyn FactoryImpl>, conn: Connection) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            conn,
        }
    }
}

/// Register a [`FactoryImpl`] on the D-Bus connection.
///
/// The factory is registered at `/org/freedesktop/IBus/Factory` so the
/// ibus-daemon can discover it and request engine instances.
pub async fn register(conn: &Connection, impl_: Box<dyn FactoryImpl>) -> Result<()> {
    let path = ObjectPath::try_from(FACTORY_PATH)
        .map_err(|e| Error::Connection(format!("Invalid factory path: {}", e)))?;

    let factory = Factory::new(impl_, conn.clone());
    conn.object_server()
        .at(path, factory)
        .await
        .map(|_| ())
        .map_err(|e| Error::Connection(format!("Failed to register factory: {}", e)))
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl Factory {
    pub async fn create_engine(&mut self, engine_name: &str) -> zbus::fdo::Result<OwnedObjectPath> {
        factory_log(&format!("create_engine called for: {}", engine_name));
        let engine_impl = {
            let mut inner = self.inner.lock().await;
            inner
                .create_engine(engine_name)
                .await
                .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to create engine: {}", e)))?
        };

        let object_path_str = next_engine_path();
        let object_path = OwnedObjectPath::try_from(object_path_str.as_str())
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid object path: {}", e)))?;

        let signal_ctxt = SignalEmitter::new(&self.conn, object_path.clone()).map_err(|e| {
            zbus::fdo::Error::Failed(format!("Failed to create signal context: {}", e))
        })?;
        let handle = EngineHandle::new(signal_ctxt.into_owned());

        let engine = Engine::new(engine_impl, handle);

        let is_new = self
            .conn
            .object_server()
            .at(object_path.clone(), engine)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to register engine: {}", e)))?;
        if !is_new {
            return Err(zbus::fdo::Error::Failed(format!(
                "Engine path {} already occupied",
                object_path_str
            )));
        }

        Ok(object_path)
    }

    pub async fn destroy_engine(&mut self, engine_name: &str) -> zbus::fdo::Result<()> {
        let mut inner = self.inner.lock().await;
        inner
            .destroy_engine(engine_name)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to destroy engine: {}", e)))
    }
}
