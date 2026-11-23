//! Servo implementation of the Moonlight EngineBackend.
//!
//! This is the only crate in the workspace permitted to depend directly on `servo`.

use euclid::Scale;
use ml_engine::{EngineBackend, EngineError, HostEvent, HostEventLoopWaker};
use raw_window_handle::{DisplayHandle, WindowHandle};
use servo::{
    EventLoopWaker, RenderingContext, Servo, ServoBuilder, WebView, WebViewBuilder,
    WebViewDelegate, WindowRenderingContext,
};
use std::rc::Rc;
use std::sync::Arc;
use url::Url;

/// Event loop waker bridging Servo's internal event loop with the host event loop.
#[derive(Clone)]
struct ServoWaker(Arc<dyn HostEventLoopWaker>);

impl EventLoopWaker for ServoWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        self.0.wake(HostEvent::Wake);
    }
}

/// WebView delegate forwarding frame readiness notifications to the host.
struct ServoDelegate {
    waker: Arc<dyn HostEventLoopWaker>,
}

impl WebViewDelegate for ServoDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        self.waker.wake(HostEvent::FrameReady);
    }
}

/// Engine backend implementation driving the Servo browser engine.
pub struct ServoEngineBackend {
    servo: Option<Servo>,
    rendering_context: Option<Rc<WindowRenderingContext>>,
    webview: Option<WebView>,
    pending_url: Option<Url>,
}

impl ServoEngineBackend {
    /// Create an unattached Servo backend.
    pub fn new() -> Self {
        // Install rustls crypto provider safely if not already initialized
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        Self {
            servo: None,
            rendering_context: None,
            webview: None,
            pending_url: None,
        }
    }
}

impl Default for ServoEngineBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBackend for ServoEngineBackend {
    fn attach_window(
        &mut self,
        display_handle: DisplayHandle<'_>,
        window_handle: WindowHandle<'_>,
        width: u32,
        height: u32,
        scale_factor: f64,
        waker: Box<dyn HostEventLoopWaker>,
    ) -> Result<(), EngineError> {
        let size = winit::dpi::PhysicalSize::new(width, height);

        let rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, size)
                .map_err(|e| EngineError::InitializationFailed(format!("{e:?}")))?,
        );

        let _ = rendering_context.make_current();

        let arc_waker: Arc<dyn HostEventLoopWaker> = Arc::from(waker);

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(ServoWaker(arc_waker.clone())))
            .build();

        let delegate = Rc::new(ServoDelegate { waker: arc_waker });

        let initial_url = self
            .pending_url
            .take()
            .unwrap_or_else(|| Url::parse("about:blank").expect("about:blank is valid"));

        let webview = WebViewBuilder::new(&servo, rendering_context.clone())
            .url(initial_url)
            .hidpi_scale_factor(Scale::new(scale_factor as f32))
            .delegate(delegate)
            .build();

        self.servo = Some(servo);
        self.rendering_context = Some(rendering_context);
        self.webview = Some(webview);

        Ok(())
    }

    fn load_url(&mut self, url: &Url) -> Result<(), EngineError> {
        if let Some(webview) = &self.webview {
            webview.load(url.clone());
        } else {
            self.pending_url = Some(url.clone());
        }
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if let Some(webview) = &self.webview {
            webview.resize(winit::dpi::PhysicalSize::new(width, height));
        }
    }

    fn paint(&mut self) {
        if let Some(webview) = &self.webview {
            webview.paint();
        }
    }

    fn present(&mut self) {
        if let Some(context) = &self.rendering_context {
            context.present();
        }
    }

    fn spin_event_loop(&mut self) {
        if let Some(servo) = &self.servo {
            servo.spin_event_loop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servo_backend_creation() {
        let mut backend = ServoEngineBackend::new();
        let url = Url::parse("about:blank").expect("valid URL");
        backend.load_url(&url).expect("load success");
        assert_eq!(backend.pending_url, Some(url));
    }
}
