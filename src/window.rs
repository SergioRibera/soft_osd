use std::{collections::HashMap, num::NonZero, sync::Arc};

use parking_lot::Mutex;
use raqote::DrawTarget;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    monitor::VideoModeHandle,
    platform::{
        wayland::{
            Anchor, Layer, MonitorHandleExtWayland, Region, WindowAttributesExtWayland,
            WindowExtWayland,
        },
        x11::WindowAttributesExtX11,
    },
    window::{WindowAttributes, WindowId},
};

use crate::{
    app::App,
    buffer::{is_wayland, new_buffer, Buffer, BufferInterface},
};
use config::{Config, OsdPosition};

pub(crate) trait AppTy: App + Sized + Send + Sync {}

impl<T: App + Sized + Send + Sync> AppTy for T {}

pub struct Window<T: AppTy> {
    width: u32,
    height: u32,
    position: OsdPosition,

    context: DrawTarget,
    render: Arc<Mutex<T>>,
    windows: HashMap<WindowId, WindowState>,

    // Inputs
    // region: WlRegion,
    active_input: bool,
    safe_left: i32,
    max_width: i32,
    safe_area: Option<Region>,
    passthrought_area: Option<Region>,
    // Variables para rastrear el gesto
    touches: HashMap<i32, (Option<(f64, f64)>, Option<(f64, f64)>)>, // (start_position, end_position)
}

fn create_window<T: AppTy>(
    app: &Window<T>,
    event_loop: &dyn ActiveEventLoop,
    window_attrs: WindowAttributes,
    scale_factor: f64,
    monitor_mode: VideoModeHandle,
) -> Option<WindowState> {
    let LogicalSize::<u32> {
        width: sw,
        height: sh,
    } = monitor_mode.size().to_logical(scale_factor);
    let w = app.width;
    let h = app.height;
    let (x, y) = match app.position {
        OsdPosition::Top => ((sw as u32 / 2) - w / 2, 0),
        OsdPosition::Left => (0, (sh as u32 / 2) - h / 2),
        OsdPosition::Right => (sw as u32 - w, (sh as u32 / 2) - h / 2),
        OsdPosition::Bottom => ((sw as u32 / 2) - w / 2, sh as u32 - h),
    };
    println!(
        "Screen({:?}): ({sw}, {sh}) => {:?} ({x}, {y}, {w}, {h})",
        monitor_mode.monitor().name(),
        app.position
    );

    let window_attrs = if is_wayland() {
        window_attrs
            .with_anchor(Anchor::LEFT | Anchor::TOP | Anchor::RIGHT)
            .with_layer(Layer::Overlay)
            .with_margin(y as i32, x as i32, 0, x as i32)
            .with_region(LogicalPosition::new(0, 0), LogicalSize::new(0, 0))
            .with_output(monitor_mode.monitor().native_id())
    } else {
        window_attrs.with_position(LogicalPosition::new(x, y))
    };

    Some(WindowState::new(
        app,
        event_loop.create_window(window_attrs).unwrap(),
    ))
}

impl<T: AppTy> Window<T> {
    pub fn run(render: Arc<Mutex<T>>, config: Config) {
        let Config { window, .. } = &config;
        let config::Window {
            position,
            width,
            height,
            radius,
            ..
        } = window.clone().unwrap_or_default();
        let width = width.unwrap_or(600);
        let height = height.unwrap_or(80);
        let radius = radius.unwrap_or(100) as f32;
        let safe_left = (radius * 2.0) - 20.0;
        let max_width = width as f32 - (radius * 3.7);
        let (width, height) = match position {
            OsdPosition::Bottom | OsdPosition::Top => (width, height),
            OsdPosition::Left | OsdPosition::Right => (height, width),
        };
        let event_loop = EventLoop::new().unwrap();
        let windows = HashMap::with_capacity(4);

        let context = DrawTarget::new(width as i32, height as i32);

        let mut app = Self {
            width,
            height,
            render,
            context,
            windows,
            position,
            active_input: false,
            touches: HashMap::new(),
            safe_area: None,
            passthrought_area: None,
            max_width: max_width as i32,
            safe_left: safe_left as i32,
        };

        event_loop.run_app(&mut app).unwrap();
    }

    pub fn draw(&mut self) -> bool {
        let mut render = self.render.lock();
        let show = render.show();

        // Draw to the window:
        self.context.clear(raqote::SolidSource {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        });

        if show {
            render.draw(&mut self.context);
        }

        show
    }
}

impl<T: AppTy> ApplicationHandler for Window<T> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_surface_size(LogicalSize::new(self.width, self.height))
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop);

        #[cfg(target_os = "linux")]
        match std::env::var("X11_VISUAL_ID") {
            Ok(visual_id_str) => {
                // info!("Using X11 visual id {visual_id_str}");
                let visual_id = visual_id_str.parse().unwrap();
                window_attributes = window_attributes.with_x11_visual(visual_id);
            }
            Err(_) => println!("Set the X11_VISUAL_ID env variable to request specific X11 visual"),
        }
        #[cfg(target_os = "linux")]
        match std::env::var("X11_SCREEN_ID") {
            Ok(screen_id_str) => {
                // info!("Placing the window on X11 screen {screen_id_str}");
                let screen_id = screen_id_str.parse().unwrap();
                window_attributes = window_attributes.with_x11_screen(screen_id);
            }
            Err(_) => println!(
                "Set the X11_SCREEN_ID env variable to place the window on non-default screen"
            ),
        }

        for (i, screen) in event_loop.available_monitors().into_iter().enumerate() {
            let Some(mode) = screen.current_video_mode() else {
                continue;
            };
            let window_attributes = window_attributes.clone();
            let Some(window_state) = create_window(
                self,
                event_loop,
                window_attributes
                    .with_title(format!("__sosd_{}", screen.name().unwrap_or(i.to_string()))),
                screen.scale_factor(),
                mode,
            ) else {
                continue;
            };
            let window_id = window_state.window.id();
            if self.safe_area.is_none() {
                self.safe_area.replace(
                    window_state
                        .window
                        .create_region(
                            LogicalPosition::new(self.safe_left, 0),
                            LogicalSize::new(self.max_width, self.height as i32),
                        )
                        .unwrap(),
                );
                self.passthrought_area.replace(
                    window_state
                        .window
                        .create_region(LogicalPosition::new(0, 0), LogicalSize::new(0, 0))
                        .unwrap(),
                );
            }
            println!("Created new window with id={window_id:?}");
            self.windows.insert(window_id, window_state);
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.windows.contains_key(&window_id) {
            return;
        }
        let can_show = self.draw();
        let window = self.windows.get_mut(&window_id).unwrap();

        match event {
            // WindowEvent::Destroyed => todo!(),
            WindowEvent::RedrawRequested => {
                window.draw(self.context.get_data());
                window.window.request_redraw();
            }
            e => {
                self.render.lock().event(e);
            }
        }

        // If the logic is collapsed not works :(
        #[allow(clippy::collapsible_if)]
        if can_show {
            if !self.active_input {
                self.active_input = true;
                for window in self.windows.values_mut() {
                    window.window.set_region(self.safe_area.as_ref());
                }
                println!("Active input");
            }
        } else {
            if self.active_input {
                self.active_input = false;
                for window in self.windows.values_mut() {
                    window.window.set_region(self.passthrought_area.as_ref());
                }
                println!("Disable input");
            }
        }
    }
}

/// State of the window.
struct WindowState {
    /// Render surface.
    ///
    /// NOTE: This surface must be dropped before the `Window`.
    pub buffer: Buffer<Arc<dyn winit::window::Window>>,
    /// The actual winit Window.
    pub window: Arc<dyn winit::window::Window>,
}

impl WindowState {
    fn new<T: AppTy>(app: &Window<T>, window: Box<dyn winit::window::Window>) -> Self {
        let window: Arc<dyn winit::window::Window> = Arc::from(window);

        // SAFETY: the surface is dropped before the `window` which provided it with handle, thus
        // it doesn't outlive it.
        let mut buffer = new_buffer(window.clone()).unwrap();
        buffer
            .resize(
                NonZero::new(app.width).unwrap(),
                NonZero::new(app.height.into()).unwrap(),
            )
            .unwrap();

        let state = Self { buffer, window };

        state
    }

    fn draw(&mut self, buff: &[u32]) {
        let buffer = self.buffer.buffer_mut().unwrap();

        buffer.copy_from_slice(buff);

        self.window.pre_present_notify();
        self.buffer.present().unwrap();
    }
}
