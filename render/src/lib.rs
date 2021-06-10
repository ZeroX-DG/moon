mod frame;
mod loader;
mod page;
mod paint;
mod renderer;

use paint::Bitmap;
use renderer::{Renderer, RendererInitializeParams};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn render_once(html: String, size: (u32, u32)) -> Option<Bitmap> {
    let mut renderer = Renderer::new().await;

    renderer.initialize(RendererInitializeParams { viewport: size });

    renderer.load_html(html);

    renderer.paint().await;

    renderer.output().clone()
}
