//! Servo implementation of the Moonlight EngineBackend.
//!
//! This is the only crate in the workspace permitted to depend directly on `servo`.

use ml_engine::EngineBackend;
use url::Url;

pub struct ServoEngineBackend;

impl ServoEngineBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ServoEngineBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBackend for ServoEngineBackend {
    fn load_url(&mut self, _url: &Url) {
        // To be connected to Servo WebView in F0 prototype spike
    }
}
