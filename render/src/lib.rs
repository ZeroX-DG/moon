mod page;
mod frame;
mod paint;
mod loader;
mod renderer;

use renderer::{Renderer, RendererInitializeParams};
use paint::Bitmap;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn render_once(html: String, css: String, size: (u32, u32)) -> Option<Bitmap> {
    let mut renderer = Renderer::new().await;

    renderer.initialize(RendererInitializeParams {
        viewport: size
    });

    renderer.load_html(html);
    renderer.load_css(css);

    renderer.paint().await;

    renderer.output().clone()
}

