use super::page::Page;
use flume::Receiver;
use shared::primitive::Size;
use url::Url;

pub enum Event {
    ViewportResize(Size),
    LoadHTML {
        html: String,
        base_url: Url,
    },
}

pub struct RenderEngine<'a> {
    page: Page<'a>
}

impl<'a> RenderEngine<'a> {
    pub async fn new(viewport: Size) -> RenderEngine<'a> {
        let page = Page::new(viewport).await;
        Self {
            page
        }
    }

    pub async fn run(mut self, event_receiver: Receiver<Event>) -> anyhow::Result<()> {
        loop {
            let event = event_receiver.recv()?;
            self.handle_event(event).await;
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::ViewportResize(new_size) => self.page.resize(new_size).await,
            Event::LoadHTML { html, base_url } => self.page.load_html(html, base_url).await,
        }
    }
}

