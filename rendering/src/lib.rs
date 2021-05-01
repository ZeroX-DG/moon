mod page;
mod frame;
mod paint;
mod loader;

use std::env;
use page::Page;
use paint::OutputBitmap;

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

