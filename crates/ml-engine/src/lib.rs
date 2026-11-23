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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend {
        loaded: Option<Url>,
    }

    impl EngineBackend for MockBackend {
        fn load_url(&mut self, url: &Url) {
            self.loaded = Some(url.clone());
        }
    }

    #[test]
    fn test_mock_backend_load_url() {
        let mut backend = MockBackend { loaded: None };
        let url = Url::parse("moonlight://newtab").expect("valid URL");
        backend.load_url(&url);
        assert_eq!(backend.loaded, Some(url));
    }
}
