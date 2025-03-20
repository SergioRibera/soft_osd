use std::error::Error;
use std::num::NonZeroI32;

use smithay_client_toolkit::reexports::client::backend::{Backend, ObjectId};
use smithay_client_toolkit::reexports::client::globals::{registry_queue_init, GlobalListContents};
use smithay_client_toolkit::reexports::client::protocol::{wl_registry, wl_shm, wl_surface};

mod buffer;
use buffer::*;
use smithay_client_toolkit::reexports::client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use super::{BufferInterface, Rect};

struct State;

pub struct WaylandImpl<W: ?Sized> {
    qh: QueueHandle<State>,
    event_queue: EventQueue<State>,
    surface: Option<wl_surface::WlSurface>,
    shm: wl_shm::WlShm,

    buffers: Option<WaylandBuffer>,
    size: Option<(NonZeroI32, NonZeroI32)>,

    /// The pointer to the window object.
    ///
    /// This has to be dropped *after* the `surface` field, because the `surface` field implicitly
    /// borrows this.
    _window_handle: W,
}

impl<H: HasWindowHandle + HasDisplayHandle> BufferInterface<H> for WaylandImpl<H> {
    fn new(handle: H) -> Result<Self, Box<dyn Error>> {
        // Get the raw Wayland window.
        let raw = handle.window_handle()?.as_raw();
        let RawWindowHandle::Wayland(wh) = raw else {
            return Err(format!("Unsuported Window Handle: {raw:?}").into());
        };

        let raw = handle.display_handle()?.as_raw();
        let RawDisplayHandle::Wayland(w) = raw else {
            return Err(format!("Unsuported Display: {raw:?}").into());
        };

        let backend = unsafe { Backend::from_foreign_display(w.display.as_ptr().cast()) };
        let conn = Connection::from_backend(backend);
        let (globals, event_queue) =
            registry_queue_init(&conn).map_err(|_| "Failed to make round trip to server")?;
        let qh: QueueHandle<State> = event_queue.handle();
        let shm: wl_shm::WlShm = globals
            .bind(&qh, 1..=1, State)
            .map_err(|_| "Failed to instantiate Wayland Shm")?;

        let surface_id = unsafe {
            ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                wh.surface.as_ptr().cast(),
            )
            .map_err(|_| "Failed to create proxy for surface ID.")?
        };

        let surface = wl_surface::WlSurface::from_id(&conn, surface_id)
            .map_err(|_| "Failed to create proxy for surface ID.")?;
        Ok(Self {
            shm,
            event_queue,
            size: None,
            buffers: None,
            qh: qh.clone(),
            surface: Some(surface),
            _window_handle: handle,
        })
    }

    fn present_with_damage(&mut self, damage: &[Rect]) -> Result<(), Box<dyn Error>> {
        let buffer = self
            .buffers
            .as_ref()
            .expect("Must set buffer before calling `present_with_damage()`");
        let surface = self
            .surface
            .as_ref()
            .expect("Must set surface before calling `present_with_damage()`");

        buffer.attach(&surface);

        if surface.version() < 4 {
            surface.damage(0, 0, i32::MAX, i32::MAX);
        } else {
            for rect in damage {
                // Introduced in version 4, it is an error to use this request in version 3 or lower.
                let (x, y, width, height) = (|| {
                    Some((
                        i32::try_from(rect.x).ok()?,
                        i32::try_from(rect.y).ok()?,
                        i32::try_from(rect.width.get()).ok()?,
                        i32::try_from(rect.height.get()).ok()?,
                    ))
                })()
                .ok_or(format!("Damage out of range: {rect:?}"))?;
                surface.damage_buffer(x, y, width, height);
            }
        }

        surface.commit();

        _ = self.event_queue.flush();

        Ok(())
    }

    fn present(&mut self) -> Result<(), Box<dyn Error>> {
        let (width, height) = self
            .size
            .expect("Must set size of surface before calling `present()`");
        self.present_with_damage(&[Rect {
            x: 0,
            y: 0,
            // We know width/height will be non-negative
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
        }])
    }

    fn buffer_mut(&mut self) -> Result<&mut [u32], Box<dyn Error>> {
        self.buffers
            .as_mut()
            .map(|buffer| buffer.buffer_mut())
            .ok_or("Buffer not initialized".into())
    }

    fn resize(
        &mut self,
        width: std::num::NonZeroU32,
        height: std::num::NonZeroU32,
    ) -> Result<(), Box<dyn Error>> {
        let width = NonZeroI32::try_from(width)
            .map_err(|e| format!("Failed to convert U32 into I32: {e:?}"))?;
        let height = NonZeroI32::try_from(height)
            .map_err(|e| format!("Failed to convert U32 into I32: {e:?}"))?;
        self.size.replace((width, height));

        match self.buffers.as_mut() {
            Some(buffers) => {
                buffers.resize(width.into(), height.into());
            }
            None => {
                self.buffers.replace(WaylandBuffer::new(
                    &self.shm,
                    &self.qh,
                    width.into(),
                    height.into(),
                ));
            }
        }

        Ok(())
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut State,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        // Ignore globals added after initialization
    }
}

impl Dispatch<wl_shm::WlShm, Self> for State {
    fn event(
        _: &mut State,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &Self,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}
