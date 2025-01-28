use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::num::{NonZeroU16, NonZeroU32};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::ptr::{null_mut, NonNull};
use std::sync::Arc;
use std::{io, slice};

use as_raw_xcb_connection::AsRawXcbConnection;
use rustix::{mm, shm as posix_shm};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, XcbDisplayHandle,
    XcbWindowHandle,
};
use x11rb::connection::{Connection, SequenceNumber};
use x11rb::cookie::Cookie;
use x11rb::protocol::xproto::{ConnectionExt as _, ImageOrder, VisualClass, Visualid};
use x11rb::protocol::{
    shm::{self, ConnectionExt as _},
    xproto,
};
use x11rb::xcb_ffi::XCBConnection;

use super::{BufferInterface, Rect};

pub struct X11DisplayImpl<H: ?Sized> {
    connection: Arc<XCBConnection>,
    /// The window to draw to.
    window: xproto::Window,

    /// The graphics context to use when drawing.
    gc: xproto::Gcontext,

    /// The depth (bits per pixel) of the drawing context.
    depth: u8,

    /// The visual ID of the drawing context.
    visual_id: u32,

    /// The buffer we draw to.
    buffer: Buffer,

    /// Buffer has been presented.
    buffer_presented: bool,

    /// The current buffer width/height.
    size: Option<(NonZeroU16, NonZeroU16)>,

    /// Keep the window alive.
    window_handle: H,
}

/// The buffer that is being drawn to.
enum Buffer {
    /// A buffer implemented using shared memory to prevent unnecessary copying.
    Shm(ShmBuffer),

    /// A normal buffer that we send over the wire.
    Wire(Vec<u32>),
}

struct ShmBuffer {
    /// The shared memory segment, paired with its ID.
    seg: Option<(ShmSegment, shm::Seg)>,

    /// A cookie indicating that the shared memory segment is ready to be used.
    ///
    /// We can't soundly read from or write to the SHM segment until the X server is done processing the
    /// `shm::PutImage` request. However, the X server handles requests in order, which means that, if
    /// we send a very small request after the `shm::PutImage` request, then the X server will have to
    /// process that request before it can process the `shm::PutImage` request. Therefore, we can use
    /// the reply to that small request to determine when the `shm::PutImage` request is done.
    ///
    /// In this case, we use `GetInputFocus` since it is a very small request.
    ///
    /// We store the sequence number instead of the `Cookie` since we cannot hold a self-referential
    /// reference to the `connection` field.
    done_processing: Option<SequenceNumber>,
}

impl<H: HasWindowHandle + HasDisplayHandle> BufferInterface<H> for X11DisplayImpl<H> {
    fn new(handle: H) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the underlying raw window handle.
        let raw = handle.window_handle()?.as_raw();
        let window_handle = match raw {
            RawWindowHandle::Xcb(xcb) => xcb,
            RawWindowHandle::Xlib(xlib) => {
                let window = NonZeroU32::new(xlib.window as u32).expect("Window ID is zero");
                let mut xcb_window_handle = XcbWindowHandle::new(window);
                xcb_window_handle.visual_id = NonZeroU32::new(xlib.visual_id as u32);
                xcb_window_handle
            }
            _ => {
                return Err(format!("Unsupported window handle type: {raw:?}").into());
            }
        };

        let raw = handle.display_handle()?.as_raw();
        let xcb_handle = match raw {
            RawDisplayHandle::Xcb(xcb_handle) => xcb_handle,
            RawDisplayHandle::Xlib(xlib) => {
                // Convert to an XCB handle.
                let connection = xlib.display.map(|display| {
                    // Get the underlying XCB connection.
                    // SAFETY: The user has asserted that the display handle is valid.
                    unsafe {
                        let display = tiny_xlib::Display::from_ptr(display.as_ptr());
                        NonNull::new_unchecked(display.as_raw_xcb_connection()).cast()
                    }
                });

                // Construct the equivalent XCB display and window handles.
                XcbDisplayHandle::new(connection, xlib.screen)
            }
            _ => return Err(format!("Unsupported display handle type: {raw:?}").into()),
        };
        // tracing::trace!("new: window_handle={:X}", window_handle.window);
        let window = window_handle.window.get();

        // Validate the display handle to ensure we can use it.
        let connection = match xcb_handle.connection {
            Some(connection) => {
                // Wrap the display handle in an x11rb connection.
                // SAFETY: We don't own the connection, so don't drop it. We also assert that the connection is valid.
                let result =
                    unsafe { XCBConnection::from_raw_xcb_connection(connection.as_ptr(), false) };

                Arc::new(result.map_err(|_| "Failed to wrap XCB connection")?)
            }
            None => {
                // The user didn't provide an XCB connection, so create our own.
                // tracing::info!("no XCB connection provided by the user, so spawning our own");
                Arc::new(
                    XCBConnection::connect(None)
                        .map_err(|_| "Failed to spawn XCB connection")?
                        .0,
                )
            }
        };

        let tokens = {
            let geometry_token = connection
                .get_geometry(window)
                .map_err(|_| "Failed to send geometry request")?;
            let window_attrs_token = if window_handle.visual_id.is_none() {
                Some(
                    connection
                        .get_window_attributes(window)
                        .map_err(|_| "Failed to send window attributes request")?,
                )
            } else {
                None
            };

            (geometry_token, window_attrs_token)
        };

        // Create a new graphics context to draw to.
        let gc = {
            let connection = connection.clone();
            connection
                .generate_id()
                .map_err(|_| "Failed to generate GC ID")?
        };
        connection
            .create_gc(
                gc,
                window,
                &xproto::CreateGCAux::new().graphics_exposures(0),
            )
            .map_err(|_| "Failed to send GC creation request")?
            .check()
            .map_err(|_| "Failed to create GC")?;

        // Finish getting the depth of the window.
        let (geometry_reply, visual_id) = {
            let (geometry_token, window_attrs_token) = tokens;
            let geometry_reply = geometry_token
                .reply()
                .map_err(|_| "Failed to get geometry reply")?;
            let visual_id = match window_attrs_token {
                None => window_handle.visual_id.unwrap().get(),
                Some(window_attrs) => {
                    window_attrs
                        .reply()
                        .map_err(|_| "Failed to get window attributes reply")?
                        .visual
                }
            };

            (geometry_reply, visual_id)
        };

        let is_shm_available = is_shm_available(&connection);
        if !is_shm_available {
            // tracing::warn!("SHM extension is not available. Performance may be poor.");
            println!("SHM extension is not available. Performance may be poor.");
        }

        let supported_visuals = supported_visuals(&connection);

        if !supported_visuals.contains(&visual_id) {
            return Err(format!(
                "Visual 0x{visual_id:x} does not use softbuffer's pixel format and is unsupported"
            )
            .into());
        }

        // See if SHM is available.
        let buffer = if is_shm_available {
            // SHM is available.
            Buffer::Shm(ShmBuffer {
                seg: None,
                done_processing: None,
            })
        } else {
            // SHM is not available.
            Buffer::Wire(Vec::new())
        };

        Ok(Self {
            connection: connection.clone(),
            window,
            gc,
            depth: geometry_reply.depth,
            visual_id,
            buffer,
            buffer_presented: false,
            size: None,
            window_handle: handle,
        })
    }

    fn buffer_mut(&mut self) -> Result<&mut [u32], Box<dyn Error>> {
        self.buffer.finish_wait(&self.connection).unwrap();
        Ok(unsafe { self.buffer.buffer_mut() })
    }

    fn resize(
        &mut self,
        width: std::num::NonZeroU32,
        height: std::num::NonZeroU32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Width and height should fit in u16.
        let width: NonZeroU16 = width
            .try_into()
            .map_err(|e| format!("Failed to convert U32 into U16: {e:?}"))?;
        let height: NonZeroU16 = height
            .try_into()
            .map_err(|e| format!("Failed to convert U32 into U16: {e:?}"))?;

        if self.size != Some((width, height)) {
            self.buffer_presented = false;
            self.buffer
                .resize(&self.connection, width.get(), height.get())
                .map_err(|_| "Failed to resize X11 buffer")?;

            // We successfully resized the buffer.
            self.size = Some((width, height));
        }
        Ok(())
    }

    fn present_with_damage(&mut self, damage: &[Rect]) -> Result<(), Box<dyn std::error::Error>> {
        let (surface_width, surface_height) = self
            .size
            .expect("Must set size of surface before calling `present_with_damage()`");

        match self.buffer {
            Buffer::Wire(ref wire) => {
                // This is a suboptimal strategy, raise a stink in the debug logs.
                // tracing::debug!("Falling back to non-SHM method for window drawing.");

                self.connection
                    .put_image(
                        xproto::ImageFormat::Z_PIXMAP,
                        self.window,
                        self.gc,
                        surface_width.get(),
                        surface_height.get(),
                        0,
                        0,
                        0,
                        self.depth,
                        bytemuck::cast_slice(wire),
                    )
                    .map(|c| c.ignore_error())
                    .map_err(|_| "Failed to draw image to window")?;
            }

            Buffer::Shm(ref mut shm) => {
                // If the X server is still processing the last image, wait for it to finish.
                // SAFETY: We know that we called finish_wait() before this.
                // Put the image into the window.
                if let Some((_, segment_id)) = shm.seg {
                    damage
                        .iter()
                        .try_for_each(|rect| {
                            let (src_x, src_y, dst_x, dst_y, width, height) = (|| {
                                Some((
                                    u16::try_from(rect.x).ok()?,
                                    u16::try_from(rect.y).ok()?,
                                    i16::try_from(rect.x).ok()?,
                                    i16::try_from(rect.y).ok()?,
                                    u16::try_from(rect.width.get()).ok()?,
                                    u16::try_from(rect.height.get()).ok()?,
                                ))
                            })(
                            )
                            .ok_or(format!("Cannot convert to u16"))?;
                            self.connection
                                .shm_put_image(
                                    self.window,
                                    self.gc,
                                    surface_width.get(),
                                    surface_height.get(),
                                    src_x,
                                    src_y,
                                    width,
                                    height,
                                    dst_x,
                                    dst_y,
                                    self.depth,
                                    xproto::ImageFormat::Z_PIXMAP.into(),
                                    false,
                                    segment_id,
                                    0,
                                )
                                .map(|c| c.ignore_error())
                                .map_err(|_| "Failed to draw image to window".to_string())
                        })
                        .and_then(|_| {
                            // Send a short request to act as a notification for when the X server is done processing the image.
                            shm.begin_wait(self.connection.as_ref())
                                .map_err(|_| "Failed to draw image to window".into())
                                .map(|c| c)
                        })?;
                }
            }
        }

        self.buffer_presented = true;

        Ok(())
    }

    fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
}

impl ShmBuffer {
    /// Allocate a new `ShmSegment` of the given size.
    fn alloc_segment(
        &mut self,
        conn: &impl Connection,
        buffer_size: usize,
    ) -> Result<(), Box<dyn Error>> {
        // Round the size up to the next power of two to prevent frequent reallocations.
        let size = buffer_size.next_power_of_two();

        // Get the size of the segment currently in use.
        let needs_realloc = match self.seg {
            Some((ref seg, _)) => seg.size() < size,
            None => true,
        };

        // Reallocate if necessary.
        if needs_realloc {
            let new_seg = ShmSegment::new(size, buffer_size)?;
            self.associate(conn, new_seg)?;
        } else if let Some((ref mut seg, _)) = self.seg {
            seg.set_buffer_size(buffer_size);
        }

        Ok(())
    }

    /// Get the SHM buffer as a reference.
    ///
    /// # Safety
    ///
    /// `finish_wait()` must be called before this function is.
    #[inline]
    unsafe fn as_ref(&self) -> &[u32] {
        match self.seg.as_ref() {
            Some((seg, _)) => {
                let buffer_size = seg.buffer_size();

                // SAFETY: No other code should be able to access the segment.
                bytemuck::cast_slice(unsafe { &seg.as_ref()[..buffer_size] })
            }
            None => {
                // Nothing has been allocated yet.
                &[]
            }
        }
    }

    /// Get the SHM buffer as a mutable reference.
    ///
    /// # Safety
    ///
    /// `finish_wait()` must be called before this function is.
    #[inline]
    unsafe fn as_mut(&mut self) -> &mut [u32] {
        match self.seg.as_mut() {
            Some((seg, _)) => {
                let buffer_size = seg.buffer_size();

                // SAFETY: No other code should be able to access the segment.
                bytemuck::cast_slice_mut(unsafe { &mut seg.as_mut()[..buffer_size] })
            }
            None => {
                // Nothing has been allocated yet.
                &mut []
            }
        }
    }

    /// Associate an SHM segment with the server.
    fn associate(&mut self, conn: &impl Connection, seg: ShmSegment) -> Result<(), Box<dyn Error>> {
        // Register the guard.
        let new_id = conn.generate_id()?;
        conn.shm_attach_fd(new_id, seg.as_fd().try_clone_to_owned().unwrap(), true)?
            .ignore_error();

        // Take out the old one and detach it.
        if let Some((old_seg, old_id)) = self.seg.replace((seg, new_id)) {
            // Wait for the old segment to finish processing.
            self.finish_wait(conn)?;

            conn.shm_detach(old_id)?.ignore_error();

            // Drop the old segment.
            drop(old_seg);
        }

        Ok(())
    }

    /// Begin waiting for the SHM processing to finish.
    fn begin_wait(&mut self, c: &impl Connection) -> Result<(), Box<dyn Error>> {
        let cookie = c.get_input_focus()?.sequence_number();
        let old_cookie = self.done_processing.replace(cookie);
        debug_assert!(old_cookie.is_none());
        Ok(())
    }

    /// Wait for the SHM processing to finish.
    fn finish_wait(&mut self, c: &impl Connection) -> Result<(), Box<dyn Error>> {
        if let Some(done_processing) = self.done_processing.take() {
            // Cast to a cookie and wait on it.
            let cookie = Cookie::<_, xproto::GetInputFocusReply>::new(c, done_processing);
            cookie.reply()?;
        }

        Ok(())
    }
}

impl Buffer {
    /// Resize the buffer to the given size.
    fn resize(
        &mut self,
        conn: &impl Connection,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Buffer::Shm(ref mut shm) => shm.alloc_segment(conn, total_len(width, height)),
            Buffer::Wire(wire) => {
                wire.resize(total_len(width, height) / 4, 0);
                Ok(())
            }
        }
    }

    /// Finish waiting for an ongoing `shm::PutImage` request, if there is one.
    fn finish_wait(&mut self, conn: &impl Connection) -> Result<(), Box<dyn Error>> {
        if let Buffer::Shm(ref mut shm) = self {
            shm.finish_wait(conn)
                .map_err(|_| "Failed to wait for X11 buffer")?;
        }

        Ok(())
    }

    /// Get a reference to the buffer.
    ///
    /// # Safety
    ///
    /// `finish_wait()` must be called in between `shm::PutImage` requests and this function.
    #[inline]
    unsafe fn buffer(&self) -> &[u32] {
        match self {
            Buffer::Shm(ref shm) => unsafe { shm.as_ref() },
            Buffer::Wire(wire) => wire,
        }
    }

    /// Get a mutable reference to the buffer.
    ///
    /// # Safety
    ///
    /// `finish_wait()` must be called in between `shm::PutImage` requests and this function.
    #[inline]
    unsafe fn buffer_mut(&mut self) -> &mut [u32] {
        match self {
            Buffer::Shm(ref mut shm) => unsafe { shm.as_mut() },
            Buffer::Wire(wire) => wire,
        }
    }
}

/// Get the length that a slice needs to be to hold a buffer of the given dimensions.
#[inline(always)]
fn total_len(width: u16, height: u16) -> usize {
    let width: usize = width.into();
    let height: usize = height.into();

    width
        .checked_mul(height)
        .and_then(|len| len.checked_mul(4))
        .unwrap_or_else(|| panic!("Dimensions are too large: ({} x {})", width, height))
}

/// Create a shared memory identifier.
fn create_shm_id() -> io::Result<OwnedFd> {
    use posix_shm::{Mode, ShmOFlags};

    let mut rng = fastrand::Rng::new();
    let mut name = String::with_capacity(23);

    // Only try four times; the chances of a collision on this space is astronomically low, so if
    // we miss four times in a row we're probably under attack.
    for i in 0..4 {
        name.clear();
        name.push_str("softbuffer-x11-");
        name.extend(std::iter::repeat_with(|| rng.alphanumeric()).take(7));

        // Try to create the shared memory segment.
        match posix_shm::shm_open(
            &name,
            ShmOFlags::RDWR | ShmOFlags::CREATE | ShmOFlags::EXCL,
            Mode::RWXU,
        ) {
            Ok(id) => {
                posix_shm::shm_unlink(&name).ok();
                return Ok(id);
            }

            Err(rustix::io::Errno::EXIST) => {
                // tracing::warn!("x11: SHM ID collision at {} on try number {}", name, i);
                println!("x11: SHM ID collision at {} on try number {}", name, i);
            }

            Err(e) => return Err(e.into()),
        };
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "failed to generate a non-existent SHM name",
    ))
}

/// Test to see if SHM is available.
fn is_shm_available(c: &impl Connection) -> bool {
    // Create a small SHM segment.
    let seg = match ShmSegment::new(0x1000, 0x1000) {
        Ok(seg) => seg,
        Err(_) => return false,
    };

    // Attach and detach it.
    let seg_id = match c.generate_id() {
        Ok(id) => id,
        Err(_) => return false,
    };

    let (attach, detach) = {
        let attach = c.shm_attach_fd(seg_id, seg.as_fd().try_clone_to_owned().unwrap(), false);
        let detach = c.shm_detach(seg_id);

        match (attach, detach) {
            (Ok(attach), Ok(detach)) => (attach, detach),
            _ => return false,
        }
    };

    // Check the replies.
    matches!((attach.check(), detach.check()), (Ok(()), Ok(())))
}

/// Collect all visuals that use softbuffer's pixel format
fn supported_visuals(c: &impl Connection) -> HashSet<Visualid> {
    // Check that depth 24 uses 32 bits per pixels
    // HACK(notgull): Also support depth 32 for transparent visuals.
    // Otherwise winit users get weird errors.
    if !c
        .setup()
        .pixmap_formats
        .iter()
        .any(|f| (f.depth == 24 || f.depth == 32) && f.bits_per_pixel == 32)
    {
        // tracing::warn!("X11 server does not have a depth 24/32 format with 32 bits per pixel");
        return HashSet::new();
    }

    // How does the server represent red, green, blue components of a pixel?
    #[cfg(target_endian = "little")]
    let own_byte_order = ImageOrder::LSB_FIRST;
    #[cfg(target_endian = "big")]
    let own_byte_order = ImageOrder::MSB_FIRST;
    let expected_masks = if c.setup().image_byte_order == own_byte_order {
        (0xff0000, 0xff00, 0xff)
    } else {
        // This is the byte-swapped version of our wished-for format
        (0xff00, 0xff0000, 0xff000000)
    };

    c.setup()
        .roots
        .iter()
        .flat_map(|screen| {
            screen
                .allowed_depths
                .iter()
                .filter(|depth| depth.depth == 24 || depth.depth == 32)
                .flat_map(|depth| {
                    depth
                        .visuals
                        .iter()
                        .filter(|visual| {
                            // Ignore grayscale or indexes / color palette visuals
                            visual.class == VisualClass::TRUE_COLOR
                                || visual.class == VisualClass::DIRECT_COLOR
                        })
                        .filter(|visual| {
                            // Colors must be laid out as softbuffer expects
                            expected_masks == (visual.red_mask, visual.green_mask, visual.blue_mask)
                        })
                        .map(|visual| visual.visual_id)
                })
        })
        .collect()
}

struct ShmSegment {
    id: File,
    ptr: NonNull<i8>,
    size: usize,
    buffer_size: usize,
}

// SAFETY: We respect Rust's mutability rules for the inner allocation.
unsafe impl Send for ShmSegment {}

impl ShmSegment {
    /// Create a new `ShmSegment` with the given size.
    fn new(size: usize, buffer_size: usize) -> io::Result<Self> {
        assert!(size >= buffer_size);

        // Create a shared memory segment.
        let id = File::from(create_shm_id()?);

        // Set its length.
        id.set_len(size as u64)?;

        // Map the shared memory to our file descriptor space.
        let ptr = NonNull::new(unsafe {
            mm::mmap(
                null_mut(),
                size,
                mm::ProtFlags::READ | mm::ProtFlags::WRITE,
                mm::MapFlags::SHARED,
                &id,
                0,
            )?
        })
        .ok_or(io::Error::new(
            io::ErrorKind::Other,
            "unexpected null when mapping SHM segment",
        ))?
        .cast();

        Ok(Self {
            id,
            ptr,
            size,
            buffer_size,
        })
    }

    /// Get this shared memory segment as a reference.
    ///
    /// # Safety
    ///
    /// One must ensure that no other processes are writing to this memory.
    unsafe fn as_ref(&self) -> &[i8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    /// Get this shared memory segment as a mutable reference.
    ///
    /// # Safety
    ///
    /// One must ensure that no other processes are reading from or writing to this memory.
    unsafe fn as_mut(&mut self) -> &mut [i8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }

    /// Set the size of the buffer for this shared memory segment.
    fn set_buffer_size(&mut self, buffer_size: usize) {
        assert!(self.size >= buffer_size);
        self.buffer_size = buffer_size
    }

    /// Get the size of the buffer for this shared memory segment.
    fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Get the size of this shared memory segment.
    fn size(&self) -> usize {
        self.size
    }
}

impl AsFd for ShmSegment {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.id.as_fd()
    }
}

impl Drop for ShmSegment {
    fn drop(&mut self) {
        unsafe {
            // Unmap the shared memory segment.
            mm::munmap(self.ptr.as_ptr().cast(), self.size).ok();
        }
    }
}
