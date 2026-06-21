use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine::trait_::EngineImpl;

pub(crate) struct Engine {
    pub(crate) inner: Arc<Mutex<Box<dyn EngineImpl>>>,
}

impl Engine {
    pub fn new(inner: Box<dyn EngineImpl>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn shared_inner(&self) -> Arc<Mutex<Box<dyn EngineImpl>>> {
        self.inner.clone()
    }
}
