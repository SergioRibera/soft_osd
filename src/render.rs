use std::num::NonZeroUsize;
use std::thread::yield_now;

use vello::kurbo::Affine;
use vello::peniko::Color;
use vello::skrifa::color::Transform;
use vello::util::RenderContext;
use vello::wgpu::TextureFormat;
use vello::{Renderer, RendererOptions, Scene};

use crate::window::Window;

pub struct Render {
    width: u32,
    height: u32,
    scene: Scene,
    ctx: RenderContext,
    renderer: Renderer,
}

impl Default for Render {
    fn default() -> Self {}
}

impl Render {
    pub fn new<T>(window: Window<T>, width: u32, height: u32) -> Self {
        let mut ctx = RenderContext::new();
        let surface =
            ctx.create_surface(window, width, height, vello::wgpu::PresentMode::AutoVsync);
        let mut renderer = Renderer::new(
            device,
            RendererOptions {
                surface_format: Some(TextureFormat::Rgba8Uint),
                antialiasing_support: vello::AaSupport::area_only(),
                use_cpu: false,
                num_init_threads: Some(NonZeroUsize::new(1)),
            },
        )
        .expect("Got non-Send/Sync error from creating renderer");
        let mut scene = Scene::new();
        Self {
            width,
            height,
            scene,
            renderer,
            ctx,
        }
    }
}
