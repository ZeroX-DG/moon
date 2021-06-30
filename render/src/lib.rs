mod frame;
mod loader;
mod page;
mod renderer;

use gfx::Bitmap;
use renderer::{Renderer, RendererInitializeParams};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn render_once(html: String, size: (u32, u32)) -> Bitmap {
    let mut renderer = Renderer::new().await;

    renderer.initialize(RendererInitializeParams { viewport: size });

    renderer.load_html(html);

    renderer.paint();

    renderer.output().await
}
