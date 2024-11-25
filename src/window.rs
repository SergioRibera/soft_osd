use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use raqote::DrawTarget;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_shm, delegate_touch, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::EventLoop,
        calloop_wayland_source::WaylandSource,
        protocols_wlr::layer_shell::v1::client::{
            zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
            zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{touch::TouchHandler, Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        xdg::window::{self, WindowConfigure, WindowHandler},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_region::{self, WlRegion},
        wl_registry::WlRegistry,
        wl_seat, wl_shm,
        wl_surface::{self, WlSurface},
    },
    Connection, Dispatch, QueueHandle,
};

use crate::{
    app::{App, AppMessage},
    config::{Config, OsdPosition},
};

pub(crate) trait AppTy: App + Sized + Send + Sync {}

impl<T: App + Sized + Send + Sync> AppTy for T {}

pub struct Window<T: AppTy> {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    screen: Option<(i32, i32)>,
    layers: HashMap<WlOutput, WlSurface>,

    context: DrawTarget,
    render: Arc<Mutex<T>>,

    // Inputs
    region: WlRegion,
    active_input: bool,
    // Variables para rastrear el gesto
    touches: HashMap<i32, (Option<(f64, f64)>, Option<(f64, f64)>)>, // (start_position, end_position)
}

fn set_pos(
    (w, h): (u32, u32),
    layer: &ZwlrLayerSurfaceV1,
    position: OsdPosition,
    screen: Option<&(i32, i32)>,
) {
    let Some(&(sw, sh)) = screen else {
        return;
    };
    let (x, y) = match position {
        OsdPosition::Top => ((sw as u32 / 2) - w / 2, 0),
        OsdPosition::Left => (0, (sh as u32 / 2) - h / 2),
        OsdPosition::Right => (sw as u32 - w, (sh as u32 / 2) - h / 2),
        OsdPosition::Bottom => ((sw as u32 / 2) - w / 2, sh as u32 - h),
    };
    println!("Screen: ({sw}, {sh}) => {position:?} ({x}, {y}, {w}, {h})");
    layer.set_layer(Layer::Top);
    layer.set_anchor(Anchor::Left | Anchor::Top);
    layer.set_size(w as u32, h as u32);
    layer.set_margin(y as i32, 0, 0, x as i32);
}

#[derive(Debug)]
struct BaseState;

// so interesting, it is just need to invoke once, it just used to get the globals
impl Dispatch<WlRegistry, GlobalListContents> for BaseState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T: AppTy + 'static> Window<T> {
    pub fn run(render: Arc<Mutex<T>>, config: Config) {
        let &Config {
            position,
            width,
            height,
            ..
        } = &config;
        let (width, height) = match position {
            OsdPosition::Bottom | OsdPosition::Top => (width, height),
            OsdPosition::Left | OsdPosition::Right => (height, width),
        };
        let conn = Connection::connect_to_env().unwrap();
        let (globals, _) = registry_queue_init::<BaseState>(&conn).unwrap();
        let mut event_queue = conn.new_event_queue::<Self>();
        let qh = event_queue.handle();

        let compositor = globals
            .bind::<WlCompositor, _, ()>(&qh, 1..=5, ())
            .expect("wl_compositor is not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

        let region = compositor.create_region(&qh, ());
        region.add(0, 0, 0, 0);

        let pool = SlotPool::new((width as usize) * (height as usize) * 4, &shm)
            .expect("Failed to create pool");
        let context = DrawTarget::new(width as i32, height as i32);
        let mut window = Self {
            shm,
            pool,
            width,
            height,
            render,
            region,
            context,
            screen: None,
            active_input: false,
            first_configure: true,
            layers: HashMap::new(),
            touches: HashMap::new(),
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),
        };

        event_queue.roundtrip(&mut window).unwrap();
        // event_queue.blocking_dispatch(&mut window).unwrap();

        for o in window.output_state.outputs() {
            let Some(output) = window.output_state.info(&o) else {
                continue;
            };
            let Some((sw, sh)) = output.logical_size else {
                continue;
            };
            let wl_surface = compositor.create_surface(&qh, ());
            let layer_shell = globals
                .bind::<ZwlrLayerShellV1, _, ()>(&qh, 3..=4, ())
                .unwrap();
            let layer = layer_shell.get_layer_surface(
                &wl_surface,
                Some(&o),
                Layer::Overlay,
                "sosd".to_owned(),
                &qh,
                (),
            );
            set_pos((width, height), &layer, position, Some(&(sw, sh)));
            wl_surface.damage(0, 0, width as i32, height as i32);
            wl_surface.frame(&qh, wl_surface.clone());
            wl_surface.commit();

            window.layers.insert(o, wl_surface);
        }

        // let mut event_loop: EventLoop<Self> =
        //     EventLoop::try_new().expect("Failed to initialize the event loop");

        // WaylandSource::new(conn.clone(), event_queue)
        //     .insert(event_loop.handle())
        //     .expect("Failed to init wayland source");

        loop {
            match event_queue.blocking_dispatch(&mut window) {
                Ok(_) => println!("Post render"),
                Err(e) => {
                    eprintln!("Error during dispatch: {:?}", e);
                    // break; // O decide cómo manejar el error
                }
            }
            // event_loop
            //     .dispatch(Duration::from_millis(1), &mut window)
            //     .unwrap();
            // event_queue.blocking_dispatch(&mut window).unwrap();
            println!("Post render");
        }
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        println!("Pre lock render");
        let Ok(mut render) = self.render.lock() else {
            println!("Failed lock render");
            return;
        };

        println!("Show");
        let show = render.show();
        let width = self.width as i32;
        let height = self.height as i32;
        let stride = width * 4;

        // Draw to the window:
        self.context.clear(raqote::SolidSource {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        });

        if show {
            println!("Render");
            render.draw(&mut self.context);
            println!("Draw");
        }
        for (_, layer) in self.layers.iter_mut() {
            if show {
                if !self.active_input {
                    println!("active input region");
                    layer.set_input_region(None);
                }
                println!("Buffer");
                let (buffer, canvas) = self
                    .pool
                    .create_buffer(
                        width as i32,
                        height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");

                println!("Assert");
                assert_eq!(canvas.len(), self.context.get_data_u8().len());
                canvas.copy_from_slice(self.context.get_data_u8());
                println!("Canvas");

                // layer.damage_buffer(0, 0, width as i32, height as i32);
                // layer.frame(qh, layer.clone());

                // Adjuntar y confirmar el buffer
                // layer.attach(Some(buffer.wl_buffer()), 0, 0);
                println!("Attach");
                buffer.attach_to(layer).unwrap();
                // buffer.attach_to(layer).expect("buffer attach");
                println!("Damage");
                layer.damage_buffer(0, 0, width, height);
            } else {
                if self.active_input {
                    println!("disable input region");
                    layer.set_input_region(Some(&self.region));
                }
            }
            println!("Frame");
            layer.frame(qh, layer.clone());
            println!("Commit");
            layer.commit();
            println!("Post Commit");
        }

        // Actualizar el estado de captura de entradas
        self.active_input = show;
    }
}

impl<T: AppTy + 'static> CompositorHandler for Window<T> {
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

impl<T: AppTy + 'static> OutputHandler for Window<T> {
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

impl<T: AppTy + 'static> LayerShellHandler for Window<T> {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        // NOTE: we not need close (?
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

impl<T: AppTy + 'static> SeatHandler for Window<T> {
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

impl<T: AppTy + 'static> ShmHandler for Window<T> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<T: AppTy + 'static> WindowHandler for Window<T> {
    fn request_close(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &window::Window,
    ) {
        // NOTE: we not need close (?
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

impl<T: AppTy + 'static> ProvidesRegistryState for Window<T> {
    registry_handlers![@<T: AppTy> OutputState, SeatState];

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

impl<T: AppTy + 'static> Dispatch<WlRegion, ()> for Window<T> {
    fn event(
        state: &mut Self,
        proxy: &WlRegion,
        event: <WlRegion as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!("Region Event: {event:?}");
    }
}

// TODO: test implementation and flow
impl<T: AppTy + 'static> TouchHandler for Window<T> {
    fn down(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
        _serial: u32,
        _time: u32,
        _surface: wl_surface::WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        // Registrar posición inicial del toque
        self.touches.insert(id, (Some(position), None));
        println!("Touch down at position: {position:?} with ID: {id}");
    }

    fn up(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
        _serial: u32,
        _time: u32,
        id: i32,
    ) {
        // Verificar si el toque está en el mapa
        if let Some((Some(start), Some(end))) = self.touches.get(&id) {
            let delta_y = start.1 - end.1; // Diferencia vertical
            let delta_x = (start.0 - end.0).abs(); // Diferencia horizontal

            if delta_y > 50.0 && delta_x < 30.0 {
                // Gesto hacia arriba detectado
                println!("Swipe up detected for ID: {id}! DeltaY: {delta_y}");
                self.render.lock().unwrap().update(AppMessage::Close);
            } else {
                println!("Not a swipe up for ID: {id}: DeltaY: {delta_y}, DeltaX: {delta_x}");
            }
        }

        // Remover el toque del mapa
        self.touches.remove(&id);
    }

    fn motion(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
        _time: u32,
        id: i32,
        position: (f64, f64),
    ) {
        // Actualizar la posición final del toque
        if let Some((_, end)) = self.touches.get_mut(&id) {
            *end = Some(position);
        }
        println!("Touch motion at position: {position:?} for ID: {id}");
    }

    fn shape(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
        _id: i32,
        _major: f64,
        _minor: f64,
    ) {
        // Este método no requiere cambios para esta lógica
    }

    fn orientation(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
        _id: i32,
        orientation: f64,
    ) {
        println!("Orientation: {orientation}");
    }

    fn cancel(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &wayland_client::protocol::wl_touch::WlTouch,
    ) {
        println!("Cancel Touch");

        // Limpiar todos los toques en caso de cancelación
        self.touches.clear();
    }
}

impl<T: AppTy + 'static> Dispatch<WlCompositor, ()> for Window<T> {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: <WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T: AppTy + 'static> Dispatch<WlSurface, ()> for Window<T> {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: <WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T: AppTy + 'static> Dispatch<ZwlrLayerShellV1, ()> for Window<T> {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerShellV1,
        event: <ZwlrLayerShellV1 as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl<T: AppTy + 'static> Dispatch<ZwlrLayerSurfaceV1, ()> for Window<T> {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrLayerSurfaceV1,
        _event: <ZwlrLayerSurfaceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

delegate_compositor!(@<T: AppTy + 'static> Window<T>);
delegate_output!(@<T: AppTy + 'static> Window<T>);
delegate_shm!(@<T: AppTy + 'static> Window<T>);
delegate_seat!(@<T: AppTy + 'static> Window<T>);
delegate_layer!(@<T: AppTy + 'static> Window<T>);
delegate_registry!(@<T: AppTy + 'static> Window<T>);
delegate_xdg_shell!(@<T: AppTy + 'static> Window<T>);
delegate_xdg_window!(@<T: AppTy + 'static> Window<T>);
delegate_touch!(@<T: AppTy + 'static> Window<T>);
