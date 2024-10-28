use std::convert::TryInto;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        xdg::window::{self, WindowConfigure, WindowHandler},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_seat, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};

use crate::{
    app::App,
    config::{Config, OsdPosition},
};

pub struct Window<T: App + Default> {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    screen: Option<(i32, i32)>,
    position: OsdPosition,
    layer: LayerSurface,

    render: T,
}

fn set_pos(
    (width, height): (u32, u32),
    layer: &LayerSurface,
    position: OsdPosition,
    screen: Option<&(i32, i32)>,
) {
    let Some(&(sw, sh)) = screen else {
        return;
    };
    let (w, h) = match position {
        OsdPosition::Bottom | OsdPosition::Top => (width, height),
        OsdPosition::Left | OsdPosition::Right => (height, width),
    };
    let mut anchor = Anchor::empty();
    let (x, y) = match position {
        OsdPosition::Top => ((sw as u32 / 2) - width / 2, 0),
        OsdPosition::Left => (0, (sh as u32 / 2) - width / 2),
        OsdPosition::Right => (sw as u32 - height, (sh as u32 / 2) - width / 2),
        OsdPosition::Bottom => ((sw as u32 / 2) - width / 2, sh as u32 - height),
    };
    println!("Screen: ({sw}, {sh}) => {position:?} ({x}, {y}, {w}, {h})");
    layer.set_anchor(Anchor::LEFT | Anchor::TOP);
    layer.set_size(w as u32, h as u32);
    layer.set_margin(y as i32, 0, 0, x as i32);
    layer.wl_surface().damage_buffer(0, 0, w as i32, h as i32);
}

impl<T: App + Default + 'static> Window<T> {
    pub fn run(
        Config {
            position,
            width,
            height,
        }: Config,
    ) {
        let conn = Connection::connect_to_env().unwrap();
        let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
        let surface = compositor.create_surface(&qh);

        let layer =
            layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("simple_layer"), None);

        let pool = SlotPool::new((width as usize) * (height as usize) * 4, &shm)
            .expect("Failed to create pool");
        let mut window = Self {
            exit: false,
            first_configure: true,
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),
            shm,
            pool,
            width,
            height,
            position,
            layer: layer.clone(),
            screen: None,
            render: Default::default(),
        };

        event_queue.roundtrip(&mut window).unwrap();

        let screen = window
            .output_state
            .info(&window.output_state.outputs().next().unwrap());

        window.screen = screen.and_then(|s| s.logical_size);

        set_pos((width, height), &layer, position, window.screen.as_ref());
        layer.commit();

        loop {
            event_queue.blocking_dispatch(&mut window).unwrap();
            if window.exit {
                break;
            }
        }
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // Draw to the window:
        {
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    let a = 0xFF;
                    let r = 0x00;
                    let g = 0x00;
                    let b = 0x00;
                    let color: i32 = (a << 24) + (r << 16) + (g << 8) + b;

                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes();
                });
        }

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}

impl<T: App + Default + 'static> CompositorHandler for Window<T> {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl<T: App + Default + 'static> OutputHandler for Window<T> {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl<T: App + Default + 'static> LayerShellHandler for Window<T> {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.width = 256;
            self.height = 256;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl<T: App + Default + 'static> SeatHandler for Window<T> {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl<T: App + Default + 'static> ShmHandler for Window<T> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<T: App + Default + 'static> WindowHandler for Window<T> {
    fn request_close(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &window::Window,
    ) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &window::Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        self.width = w.map(|w| w.get()).unwrap_or(self.width);
        self.height = h.map(|h| h.get()).unwrap_or(self.height);

        window.unset_fullscreen();

        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl<T: App + Default + 'static> ProvidesRegistryState for Window<T> {
    registry_handlers![@<T: App + Default + 'static> OutputState, SeatState];

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

delegate_compositor!(@<T: App + Default + 'static> Window<T>);
delegate_output!(@<T: App + Default + 'static> Window<T>);
delegate_shm!(@<T: App + Default + 'static> Window<T>);
delegate_seat!(@<T: App + Default + 'static> Window<T>);
delegate_layer!(@<T: App + Default + 'static> Window<T>);
delegate_registry!(@<T: App + Default + 'static> Window<T>);
delegate_xdg_shell!(@<T: App + Default + 'static> Window<T>);
delegate_xdg_window!(@<T: App + Default + 'static> Window<T>);
