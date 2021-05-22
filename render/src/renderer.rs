use super::page::Page;
use super::paint::Painter;
use super::messenger::Messenger;
use message::*;

#[derive(Debug)]
pub enum RendererError {
    ReceiveMessageError(String),
    CastingResponseError(String),
    CastingNotificationError(String),
    KernelDisconnected
}

pub struct Renderer {
    id: String,
    page: Page,
    painter: Painter,
    messenger: Messenger,
}

impl Renderer {
    pub async fn new(id: String) -> Self {
        Self {
            id,
            page: Page::new(),
            painter: Painter::new().await,
            messenger: Messenger::new()
        }
    }

    pub fn initialize(&mut self) -> Result<(), RendererError> {
        log::debug!("Sending Syn to kernel");
        self.messenger.send_notification::<Syn>(&SynParams {
            id: self.id.clone()
        })?;

        Ok(())
    }

    pub async fn run_event_loop(&mut self) {
        loop {
            let result = match self.messenger.receive() {
                Ok(BrowserMessage::Request(raw_request)) => self.handle_request(raw_request).await,
                Ok(BrowserMessage::Response(raw_response)) => self.handle_response(raw_response).await,
                Ok(BrowserMessage::Notification(raw_notification)) => self.handle_notification(raw_notification).await,
                Err(e) => Err(e)
            };

            if let Err(e) = result {
                log::error!("Renderer error encountered: {:?}", e);
                break
            }
        }
    }

    // TODO: temporary, need better naming.
    async fn redraw(&mut self) {
        self.page.paint(&mut self.painter).await;
    }

    async fn handle_response(&mut self, raw_response: RawResponse) -> Result<(), RendererError> {
        if let Some(callback) = self.messenger.get_callback(raw_response.request_id) {
            return (callback.func)(self, raw_response);
        }
        Ok(())
    }

    async fn handle_notification(&mut self, raw_notification: RawNotification) -> Result<(), RendererError> {
        match raw_notification.method.as_str() {
            SynAck::METHOD => {
                log::debug!("Received SynAck, sending Ack to kernel");
                self.messenger.send_notification::<Ack>(&SynParams {
                    id: self.id.clone()
                })
            }
            LoadFile::METHOD => {
                let params = raw_notification.cast::<LoadFile>()
                    .map_err(|e| RendererError::CastingNotificationError(
                            format!("Error while casting: {}", e.method)))?;
                log::info!("Received LoadFile notification with content type: {}", params.content_type);
                
                match params.content_type.as_str() {
                    "text/html" => self.page.load_html(params.content),
                    "text/css" => self.page.load_css(params.content),
                    _ => log::debug!("Unknown content type to load: {}", params.content_type),
                }

                self.redraw().await;
                Ok(())
            }
            _ => Ok(())
        }
    }

    async fn handle_request(&mut self, raw_request: RawRequest) -> Result<(), RendererError> {
        match raw_request.method.as_str() {
            GetRenderedBitmap::METHOD => {
                let (id, _) = raw_request.cast::<GetRenderedBitmap>().unwrap();
                
                self.messenger.send_response_ok::<GetRenderedBitmap>(id, &RenderedBitmap {
                    data: self.page.last_output_bitmap().unwrap_or(Vec::new())
                })
            }
            _ => Ok(())
        }
    }
}
