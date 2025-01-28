use std::env::var_os;
use std::error::Error;
use std::num::NonZeroU32;

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod wayland;
mod x11;

pub fn is_wayland() -> bool {
    var_os("WAYLAND_DISPLAY")
        .or(var_os("XDG_SESSION_TYPE"))
        .is_some_and(|v| {
            v.to_str()
                .unwrap_or_default()
                .to_lowercase()
                .contains("wayland")
        })
}

/// A rectangular region of the buffer coordinate space.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    /// x coordinate of top left corner
    pub x: u32,
    /// y coordinate of top left corner
    pub y: u32,
    /// width
    pub width: NonZeroU32,
    /// height
    pub height: NonZeroU32,
}

pub trait BufferInterface<H: HasWindowHandle + HasDisplayHandle>: Sized {
    fn new(window: H) -> Result<Self, Box<dyn Error>>;
    fn buffer_mut(&mut self) -> Result<&mut [u32], Box<dyn Error>>;
    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Result<(), Box<dyn Error>>;
    fn present_with_damage(&mut self, damage: &[Rect]) -> Result<(), Box<dyn Error>>;
    fn present(&mut self) -> Result<(), Box<dyn Error>>;
}

pub enum Buffer<H: HasWindowHandle + HasDisplayHandle> {
    Wayland(wayland::WaylandImpl<H>),
    X11(x11::X11DisplayImpl<H>),
}

impl<H: HasWindowHandle + HasDisplayHandle> BufferInterface<H> for Buffer<H> {
    fn new(window: H) -> Result<Self, Box<dyn Error>> {
        if is_wayland() {
            Ok(Buffer::Wayland(wayland::WaylandImpl::new(window)?))
        } else {
            Ok(Buffer::X11(x11::X11DisplayImpl::new(window)?))
        }
    }

    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Result<(), Box<dyn Error>> {
        match self {
            Buffer::Wayland(impl_) => impl_.resize(width, height),
            Buffer::X11(impl_) => impl_.resize(width, height),
        }
    }

    fn present_with_damage(&mut self, damage: &[Rect]) -> Result<(), Box<dyn Error>> {
        match self {
            Buffer::Wayland(impl_) => impl_.present_with_damage(damage),
            Buffer::X11(impl_) => impl_.present_with_damage(damage),
        }
    }

    fn present(&mut self) -> Result<(), Box<dyn Error>> {
        match self {
            Buffer::Wayland(impl_) => impl_.present(),
            Buffer::X11(impl_) => impl_.present(),
        }
    }

    fn buffer_mut(&mut self) -> Result<&mut [u32], Box<dyn Error>> {
        match self {
            Buffer::Wayland(impl_) => impl_.buffer_mut(),
            Buffer::X11(impl_) => impl_.buffer_mut(),
        }
    }
}

pub fn new_buffer<H: HasWindowHandle + HasDisplayHandle>(
    handle: H,
) -> Result<Buffer<H>, Box<dyn Error>> {
    if is_wayland() {
        Ok(Buffer::Wayland(wayland::WaylandImpl::new(handle)?))
    } else {
        Ok(Buffer::X11(x11::X11DisplayImpl::new(handle)?))
    }
}
