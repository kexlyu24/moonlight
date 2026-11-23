//! Browser shell for Moonlight (windowing, tab strip, omnibox, toolbar).

use ml_engine::{EngineBackend, HostEvent, HostEventLoopWaker};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use tracing::{error, info, warn};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId};

/// Concrete waker implementing `HostEventLoopWaker` using `winit::event_loop::EventLoopProxy`.
pub struct WinitWaker(pub EventLoopProxy<HostEvent>);

impl HostEventLoopWaker for WinitWaker {
    fn wake(&self, event: HostEvent) {
        if let Err(e) = self.0.send_event(event) {
            warn!("failed to wake host event loop: {e:?}");
        }
    }
}

/// The state of the browser application window.
pub struct BrowserApp<B: EngineBackend> {
    backend: B,
    proxy: EventLoopProxy<HostEvent>,
    initial_url: Url,
    window: Option<Window>,
}

impl<B: EngineBackend> BrowserApp<B> {
    pub fn new(backend: B, proxy: EventLoopProxy<HostEvent>, initial_url: Url) -> Self {
        Self {
            backend,
            proxy,
            initial_url,
            window: None,
        }
    }
}

impl<B: EngineBackend> ApplicationHandler<HostEvent> for BrowserApp<B> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("Moonlight")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 800.0));

        let window = match event_loop.create_window(attributes) {
            Ok(w) => w,
            Err(e) => {
                error!("failed to create window: {e:?}");
                return;
            }
        };

        let display_handle = match event_loop.display_handle() {
            Ok(h) => h,
            Err(e) => {
                error!("failed to get display handle: {e:?}");
                return;
            }
        };

        let window_handle = match window.window_handle() {
            Ok(h) => h,
            Err(e) => {
                error!("failed to get window handle: {e:?}");
                return;
            }
        };

        let size = window.inner_size();
        let scale = window.scale_factor();
        let waker = Box::new(WinitWaker(self.proxy.clone()));

        if let Err(e) = self.backend.load_url(&self.initial_url) {
            warn!("failed to load initial url: {e:?}");
        }

        if let Err(e) = self.backend.attach_window(
            display_handle,
            window_handle,
            size.width,
            size.height,
            scale,
            waker,
        ) {
            error!("failed to attach window to engine: {e:?}");
        }

        self.window = Some(window);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: HostEvent) {
        match event {
            HostEvent::Wake => {
                self.backend.spin_event_loop();
            }
            HostEvent::FrameReady => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.backend.spin_event_loop();

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, exiting application");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.backend.paint();
                self.backend.present();
            }
            WindowEvent::Resized(new_size) => {
                self.backend.resize(new_size.width, new_size.height);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Run the Moonlight browser shell with the given engine backend and initial URL.
pub fn run<B: EngineBackend + 'static>(
    backend: B,
    url: Url,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::<HostEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mut app = BrowserApp::new(backend, proxy, url);
    event_loop.run_app(&mut app)?;
    Ok(())
}
