//! EngineBackend abstraction for Moonlight.
//!
//! Provides the engine-agnostic interface through which the shell,
//! extension runtime, and network policies communicate with the browser engine.

use url::Url;

/// Generic engine backend interface.
pub trait EngineBackend {
    /// Load a URL into the primary view.
    fn load_url(&mut self, url: &Url);
}
