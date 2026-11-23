//! EngineBackend abstraction for Moonlight.
//!
//! Provides the engine-agnostic interface through which the shell,
//! extension runtime, and network policies communicate with the browser engine.

use raw_window_handle::{DisplayHandle, WindowHandle};
use std::fmt;
use url::Url;

/// Represents errors originating from the rendering engine.
#[derive(Debug)]
pub enum EngineError {
    InitializationFailed(String),
    NavigationFailed(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitializationFailed(msg) => write!(f, "engine initialization failed: {msg}"),
            Self::NavigationFailed(msg) => write!(f, "navigation failed: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

/// Events triggered by the engine directed to the hosting shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostEvent {
    /// Request the event loop to wake and spin internal engine tasks.
    Wake,
    /// A new webview frame has finished layout and is ready to be painted.
    FrameReady,
}

/// Abstract waker allowing the engine to notify the host event loop.
pub trait HostEventLoopWaker: Send + Sync {
    fn wake(&self, event: HostEvent);
}

/// Generic engine backend interface.
///
/// Any rendering engine (e.g. Servo or potential CEF fallback) must implement
/// this trait to interface with Moonlight's UI shell and subsystems.
pub trait EngineBackend {
    /// Initialize the engine with raw window handles and initial dimensions.
    fn attach_window(
        &mut self,
        display_handle: DisplayHandle<'_>,
        window_handle: WindowHandle<'_>,
        width: u32,
        height: u32,
        scale_factor: f64,
        waker: Box<dyn HostEventLoopWaker>,
    ) -> Result<(), EngineError>;

    /// Load a URL into the primary view.
    fn load_url(&mut self, url: &Url) -> Result<(), EngineError>;

    /// Navigate back in session history.
    fn go_back(&mut self);

    /// Navigate forward in session history.
    fn go_forward(&mut self);

    /// Reload the current page.
    fn reload(&mut self);

    /// Retrieve the current URL of the primary view.
    fn current_url(&self) -> Option<Url>;

    /// Notify the engine of window resize.
    fn resize(&mut self, width: u32, height: u32);

    /// Request a paint of the webview content into the rendering context.
    fn paint(&mut self);

    /// Present the painted frame to the display.
    fn present(&mut self);

    /// Spin the engine event loop to process internal messages and tasks.
    fn spin_event_loop(&mut self);

    /// Optional OpenGL context handle for shells that render UI overlays via glow.
    fn glow_context(&self) -> Option<std::sync::Arc<glow::Context>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend {
        attached: bool,
        loaded: Option<Url>,
        size: (u32, u32),
    }

    impl MockBackend {
        fn new() -> Self {
            Self {
                attached: false,
                loaded: None,
                size: (0, 0),
            }
        }
    }

    impl EngineBackend for MockBackend {
        fn attach_window(
            &mut self,
            _display_handle: DisplayHandle<'_>,
            _window_handle: WindowHandle<'_>,
            width: u32,
            height: u32,
            _scale_factor: f64,
            _waker: Box<dyn HostEventLoopWaker>,
        ) -> Result<(), EngineError> {
            self.attached = true;
            self.size = (width, height);
            Ok(())
        }

        fn load_url(&mut self, url: &Url) -> Result<(), EngineError> {
            self.loaded = Some(url.clone());
            Ok(())
        }

        fn go_back(&mut self) {}

        fn go_forward(&mut self) {}

        fn reload(&mut self) {}

        fn current_url(&self) -> Option<Url> {
            self.loaded.clone()
        }

        fn resize(&mut self, width: u32, height: u32) {
            self.size = (width, height);
        }

        fn paint(&mut self) {}

        fn present(&mut self) {}

        fn spin_event_loop(&mut self) {}
    }

    #[test]
    fn test_mock_backend_lifecycle() {
        let mut backend = MockBackend::new();
        assert!(!backend.attached);
        assert_eq!(backend.size, (0, 0));

        let url = Url::parse("moonlight://newtab").expect("valid test URL");
        backend.load_url(&url).expect("load success");
        assert_eq!(backend.loaded, Some(url));

        backend.resize(1024, 768);
        assert_eq!(backend.size, (1024, 768));
    }
}
