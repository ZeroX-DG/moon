mod page;
mod frame;
mod paint;
mod loader;
mod renderer;
mod messenger;

use page::Page;
use paint::OutputBitmap;
use renderer::Renderer;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn render_once(html: String, css: String, size: (u32, u32)) -> Option<OutputBitmap> {
    let mut painter = paint::Painter::new().await;
    let mut page = Page::new();

    page.set_size(size);

    page.load_html(html);
    page.load_css(css);

    page.paint(&mut painter).await
}

pub async fn run_event_loop(id: String) -> Result<(), String> {
    let mut renderer = Renderer::new(id).await;
    renderer.initialize()
        .map_err(|e| format!("Unexpected error while initialize renderer: {:?}", e))?;
    renderer.run_event_loop().await;
    Ok(())
}

