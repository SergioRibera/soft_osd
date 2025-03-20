use std::ffi::CStr;
use std::fs::File;
use std::os::fd::{AsFd, AsRawFd};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use memmap2::MmapMut;
use smithay_client_toolkit::reexports::client::protocol::wl_buffer::WlBuffer;
use smithay_client_toolkit::reexports::client::protocol::wl_shm_pool::WlShmPool;
use smithay_client_toolkit::reexports::client::protocol::{
    wl_buffer, wl_shm, wl_shm_pool, wl_surface,
};
use smithay_client_toolkit::reexports::client::{Connection, Dispatch, QueueHandle};

use super::State;

// Round size to use for pool for given dimensions, rounding up to power of 2
fn get_pool_size(width: i32, height: i32) -> i32 {
    ((width * height * 4) as u32).next_power_of_two() as i32
}

fn create_memfile() -> File {
    use rustix::fs::{MemfdFlags, SealFlags};

    let name = unsafe { CStr::from_bytes_with_nul_unchecked("softbuffer\0".as_bytes()) };
    let fd = rustix::fs::memfd_create(name, MemfdFlags::CLOEXEC | MemfdFlags::ALLOW_SEALING)
        .expect("Failed to create memfd to store buffer.");
    rustix::fs::fcntl_add_seals(&fd, SealFlags::SHRINK | SealFlags::SEAL)
        .expect("Failed to seal memfd.");
    File::from(fd)
}

unsafe fn map_file(file: &File) -> MmapMut {
    unsafe { MmapMut::map_mut(file.as_raw_fd()).expect("Failed to map shared memory") }
}

pub(super) struct WaylandBuffer {
    qh: QueueHandle<State>,
    pool: WlShmPool,
    pool_size: i32,
    buffer: WlBuffer,
    map: MmapMut,
    width: i32,
    height: i32,
    released: Arc<AtomicBool>,
}

impl WaylandBuffer {
    pub fn new(shm: &wl_shm::WlShm, qh: &QueueHandle<State>, width: i32, height: i32) -> Self {
        // Calculate size to use for shm pool
        let pool_size = get_pool_size(width, height);

        // Create wayland shm pool and buffer
        let tempfile = create_memfile();
        let _ = tempfile.set_len(pool_size as u64);
        let map = unsafe { map_file(&tempfile) };

        let pool = shm.create_pool(tempfile.as_fd(), pool_size, qh, ());
        let released = Arc::new(AtomicBool::new(true));
        let buffer = pool.create_buffer(
            0,
            width,
            height,
            width * 4,
            wl_shm::Format::Argb8888,
            qh,
            released.clone(),
        );

        Self {
            pool,
            pool_size,
            buffer,
            map,
            width,
            height,
            released,
            qh: qh.clone(),
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        // If size is the same, there's nothing to do
        if self.width != width || self.height != height {
            // Grow pool, if needed
            let size = ((width * height * 4) as u32).next_power_of_two() as i32;
            if size > self.pool_size {
                self.pool.resize(size);
                self.pool_size = size;
            }

            // Create buffer with correct size
            self.buffer = self.pool.create_buffer(
                0,
                width,
                height,
                width * 4,
                wl_shm::Format::Argb8888,
                &self.qh,
                self.released.clone(),
            );

            self.width = width;
            self.height = height;
        }
    }

    pub fn buffer_mut(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.map.as_mut_ptr() as *mut u32, self.len()) }
    }

    pub fn attach(&self, surface: &wl_surface::WlSurface) {
        self.released.store(false, Ordering::SeqCst);
        surface.attach(Some(&self.buffer), 0, 0);
    }

    // pub fn released(&self) -> bool {
    //     self.released.load(Ordering::SeqCst)
    // }

    fn len(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _: &mut State,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, Arc<AtomicBool>> for State {
    fn event(
        _: &mut State,
        _: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        released: &Arc<AtomicBool>,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        if let wl_buffer::Event::Release = event {
            released.store(true, Ordering::SeqCst);
        }
    }
}
