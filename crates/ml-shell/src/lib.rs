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

/// Resolve omnibox user input into a navigable URL.
///
/// Handles valid URLs, domain-like inputs (prepending `https://`),
/// and search queries (routing through DuckDuckGo).
pub fn resolve_omnibox_input(input: &str) -> Option<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(url) = Url::parse(trimmed) {
        if !url.scheme().is_empty() {
            return Some(url);
        }
    }

    if trimmed.contains('.') && !trimmed.contains(' ') {
        if let Ok(url) = Url::parse(&format!("https://{trimmed}")) {
            return Some(url);
        }
    }

    if let Ok(mut search_url) = Url::parse("https://duckduckgo.com/") {
        search_url.query_pairs_mut().append_pair("q", trimmed);
        Some(search_url)
    } else {
        None
    }
}

/// The state of the browser application window.
pub struct BrowserApp<B: EngineBackend> {
    backend: B,
    proxy: EventLoopProxy<HostEvent>,
    initial_url: Url,
    window: Option<Window>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_painter: Option<egui_glow::Painter>,
    omnibox_text: String,
    omnibox_focused: bool,
}

impl<B: EngineBackend> BrowserApp<B> {
    pub fn new(backend: B, proxy: EventLoopProxy<HostEvent>, initial_url: Url) -> Self {
        let egui_ctx = egui::Context::default();

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(0x26, 0x2C, 0x3D);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x16, 0x1A, 0x23);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x16, 0x1A, 0x23);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x1D, 0x22, 0x30);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x2C, 0x33, 0x43);
        visuals.override_text_color = Some(egui::Color32::from_rgb(0xE6, 0xE9, 0xF2));
        egui_ctx.set_visuals(visuals);

        let omnibox_text = initial_url.to_string();

        Self {
            backend,
            proxy,
            initial_url,
            window: None,
            egui_ctx,
            egui_state: None,
            egui_painter: None,
            omnibox_text,
            omnibox_focused: false,
        }
    }

    /// Navigate the browser to a user-entered URL or search query.
    pub fn navigate(&mut self, input: &str) {
        if let Some(url) = resolve_omnibox_input(input) {
            self.omnibox_text = url.to_string();
            if let Err(e) = self.backend.load_url(&url) {
                warn!("failed to navigate: {e:?}");
            }
        }
    }

    /// Read the current text displayed in the omnibox.
    pub fn omnibox_text(&self) -> &str {
        &self.omnibox_text
    }
}

impl<B: EngineBackend> Drop for BrowserApp<B> {
    fn drop(&mut self) {
        if let Some(mut painter) = self.egui_painter.take() {
            painter.destroy();
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

        let state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        self.egui_state = Some(state);

        if let Some(gl) = self.backend.glow_context() {
            match egui_glow::Painter::new(gl, "", None, false) {
                Ok(painter) => {
                    self.egui_painter = Some(painter);
                }
                Err(e) => {
                    error!("failed to create egui glow painter: {e:?}");
                }
            }
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

        if let (Some(window), Some(state)) = (&self.window, &mut self.egui_state) {
            let response = state.on_window_event(window, &event);
            if response.repaint {
                window.request_redraw();
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, exiting application");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.backend.paint();

                if let (Some(window), Some(state), Some(painter)) =
                    (&self.window, &mut self.egui_state, &mut self.egui_painter)
                {
                    let raw_input = state.take_egui_input(window);
                    let mut navigate_target = None;

                    if !self.omnibox_focused {
                        if let Some(current) = self.backend.current_url() {
                            let current_str = current.to_string();
                            if self.omnibox_text != current_str {
                                self.omnibox_text = current_str;
                            }
                        }
                    }

                    let full_output = self.egui_ctx.run(raw_input, |ctx| {
                        egui::TopBottomPanel::top("browser_toolbar")
                            .frame(
                                egui::Frame::default()
                                    .fill(egui::Color32::from_rgb(0x26, 0x2C, 0x3D))
                                    .inner_margin(egui::Margin::symmetric(8, 6)),
                            )
                            .show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;

                                    let back_btn = ui.button("⬅");
                                    if back_btn
                                        .on_hover_text(ml_i18n::text("toolbar-back"))
                                        .clicked()
                                    {
                                        self.backend.go_back();
                                    }

                                    let forward_btn = ui.button("➡");
                                    if forward_btn
                                        .on_hover_text(ml_i18n::text("toolbar-forward"))
                                        .clicked()
                                    {
                                        self.backend.go_forward();
                                    }

                                    let reload_btn = ui.button("⟳");
                                    if reload_btn
                                        .on_hover_text(ml_i18n::text("toolbar-reload"))
                                        .clicked()
                                    {
                                        self.backend.reload();
                                    }

                                    let omnibox_width = ui.available_width();
                                    let text_edit =
                                        egui::TextEdit::singleline(&mut self.omnibox_text)
                                            .hint_text(ml_i18n::text("omnibox-placeholder"))
                                            .text_color(egui::Color32::from_rgb(0xE6, 0xE9, 0xF2));

                                    let response = ui.add_sized([omnibox_width, 24.0], text_edit);
                                    self.omnibox_focused = response.has_focus();

                                    if (response.lost_focus() || response.has_focus())
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    {
                                        navigate_target = Some(self.omnibox_text.clone());
                                    }
                                });
                            });
                    });

                    if let Some(target) = navigate_target {
                        if let Some(url) = resolve_omnibox_input(&target) {
                            self.omnibox_text = url.to_string();
                            if let Err(e) = self.backend.load_url(&url) {
                                warn!("failed to navigate from omnibox: {e:?}");
                            }
                        }
                    }

                    state.handle_platform_output(window, full_output.platform_output);

                    let clipped_primitives = self
                        .egui_ctx
                        .tessellate(full_output.shapes, full_output.pixels_per_point);

                    let size = window.inner_size();
                    painter.paint_and_update_textures(
                        [size.width, size.height],
                        full_output.pixels_per_point,
                        &clipped_primitives,
                        &full_output.textures_delta,
                    );
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_omnibox_valid_urls() {
        let url = resolve_omnibox_input("https://servo.org").expect("should resolve");
        assert_eq!(url.as_str(), "https://servo.org/");

        let about = resolve_omnibox_input("about:blank").expect("should resolve");
        assert_eq!(about.as_str(), "about:blank");
    }

    #[test]
    fn test_resolve_omnibox_domain_without_scheme() {
        let url = resolve_omnibox_input("example.com").expect("should resolve");
        assert_eq!(url.as_str(), "https://example.com/");

        let path_url = resolve_omnibox_input("crates.io/crates/servo").expect("should resolve");
        assert_eq!(path_url.as_str(), "https://crates.io/crates/servo");
    }

    #[test]
    fn test_resolve_omnibox_search_query() {
        let url = resolve_omnibox_input("rust browser engine").expect("should resolve");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("duckduckgo.com"));
        assert!(url.query().unwrap_or("").contains("q=rust+browser+engine"));
    }

    #[test]
    fn test_resolve_omnibox_empty() {
        assert!(resolve_omnibox_input("").is_none());
        assert!(resolve_omnibox_input("   ").is_none());
    }

    #[test]
    fn test_fluent_strings_available() {
        assert_eq!(ml_i18n::text("toolbar-back"), "Back");
        assert_eq!(ml_i18n::text("toolbar-forward"), "Forward");
        assert_eq!(ml_i18n::text("toolbar-reload"), "Reload");
        assert_eq!(
            ml_i18n::text("omnibox-placeholder"),
            "Search DuckDuckGo or type a URL"
        );
    }
}
