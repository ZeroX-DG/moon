mod frame;
mod page;
mod renderer;

use gfx::Bitmap;
use renderer::{Renderer, RendererInitializeParams};
use shared::primitive::Size;
use url::Url;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn render_once(html: String, base_url: Url, size: Size) -> Bitmap {
    let mut renderer = Renderer::new();

    renderer.initialize(RendererInitializeParams { viewport: size });

    renderer.load_html(html, base_url);

    renderer.output()
}
