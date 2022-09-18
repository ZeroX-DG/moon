use super::page::Page;
use flume::{Receiver, Sender};
use gfx::Bitmap;
use loader::resource_loop::{request::LoadRequest, ResourceLoop};
use shared::primitive::{Point, Size};
use url::Url;

pub enum InputEvent {
    ViewportResize(Size),
    Scroll(f32),
    MouseMove(Point),
    LoadHTML { html: String, base_url: Url },
    LoadURL(Url),
    Reload,
}

pub enum OutputEvent {
    FrameRendered(Bitmap),
    TitleChanged(String),
    LoadingStarted,
    LoadingFinished,
}

pub struct RenderEngine<'a> {
    page: Page<'a>,
    resource_loop_tx: Sender<LoadRequest>,
}

impl<'a> RenderEngine<'a> {
    pub async fn new(viewport: Size) -> RenderEngine<'a> {
        let page = Page::new(viewport).await;
        let resource_loop = ResourceLoop::new();
        let resource_loop_tx = resource_loop.start_loop();
        Self {
            page,
            resource_loop_tx,
        }
    }

    pub async fn run(
        mut self,
        event_receiver: Receiver<InputEvent>,
        event_emitter: Sender<OutputEvent>,
    ) -> anyhow::Result<()> {
        loop {
            let event = event_receiver.recv()?;
            self.handle_event(event, &event_emitter).await?;
        }
    }

    async fn handle_event(
        &mut self,
        event: InputEvent,
        event_emitter: &Sender<OutputEvent>,
    ) -> anyhow::Result<()> {
        match event {
            InputEvent::ViewportResize(new_size) => {
                self.page.resize(new_size).await;
                self.emit_new_frame(event_emitter)?;
            }
            InputEvent::Scroll(delta_y) => {
                self.page.scroll(delta_y).await;
                self.emit_new_frame(event_emitter)?;
            }
            InputEvent::MouseMove(coord) => {
                self.page.handle_mouse_move(coord).await;
                self.emit_new_frame(event_emitter)?;
            }
            InputEvent::LoadHTML { html, base_url } => {
                self.emit_loading_started(event_emitter)?;
                self.page
                    .load_html(html, base_url, self.resource_loop_tx.clone())
                    .await;
                self.emit_loading_finished(event_emitter)?;
                self.emit_new_frame(event_emitter)?;
                self.emit_new_title(event_emitter)?;
            }
            InputEvent::LoadURL(url) => {
                self.emit_loading_started(event_emitter)?;
                self.page.load_url(url, self.resource_loop_tx.clone()).await;
                self.emit_loading_finished(event_emitter)?;
                self.emit_new_frame(event_emitter)?;
                self.emit_new_title(event_emitter)?;
            }
            InputEvent::Reload => {
                self.emit_loading_started(event_emitter)?;
                self.page.reload(self.resource_loop_tx.clone()).await;
                self.emit_loading_finished(event_emitter)?;
                self.emit_new_frame(event_emitter)?;
                self.emit_new_title(event_emitter)?;
            }
        }
        Ok(())
    }

    fn emit_loading_started(&self, event_emitter: &Sender<OutputEvent>) -> anyhow::Result<()> {
        event_emitter.send(OutputEvent::LoadingStarted)?;
        Ok(())
    }

    fn emit_loading_finished(&self, event_emitter: &Sender<OutputEvent>) -> anyhow::Result<()> {
        event_emitter.send(OutputEvent::LoadingFinished)?;
        Ok(())
    }

    fn emit_new_title(&self, event_emitter: &Sender<OutputEvent>) -> anyhow::Result<()> {
        event_emitter.send(OutputEvent::TitleChanged(self.page.title()))?;
        Ok(())
    }

    fn emit_new_frame(&self, event_emitter: &Sender<OutputEvent>) -> anyhow::Result<()> {
        if let Some(frame) = self.page.bitmap() {
            event_emitter.send(OutputEvent::FrameRendered(frame.clone()))?;
        }
        Ok(())
    }
}
