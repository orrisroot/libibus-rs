use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine::handle::EngineHandle;
use crate::engine::trait_::EngineImpl;

pub(crate) struct Engine {
    pub(crate) inner: Arc<Mutex<Box<dyn EngineImpl>>>,
    pub(crate) handle: EngineHandle,
}

impl Engine {
    pub fn new(inner: Box<dyn EngineImpl>, handle: EngineHandle) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            handle,
        }
    }
}
