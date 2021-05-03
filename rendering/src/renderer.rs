use super::page::Page;
use super::paint::Painter;
use super::messenger::Messenger;
use message::*;

#[derive(Debug)]
pub enum RendererError {
    ReceiveMessageError(String),
    CastingResponseError(String),
    KernelDisconnected
}

pub struct Renderer {
    page: Page,
    painter: Painter,
    messenger: Messenger,
}

impl Renderer {
    pub async fn new() -> Self {
        Self {
            page: Page::new(),
            painter: Painter::new().await,
            messenger: Messenger::new()
        }
    }

    fn handle_response(&mut self, raw_response: RawResponse) -> Result<(), RendererError> {
        if let Some(callback) = self.messenger.get_callback(raw_response.request_id) {
            return (callback.func)(self, raw_response);
        }
        Ok(())
    }

    fn handle_notification(&mut self, raw_notification: RawNotification) -> Result<(), RendererError> {
        Ok(())
    }

    fn handle_request(&mut self, raw_request: RawRequest) -> Result<(), RendererError> {
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

    pub fn run_event_loop(&mut self) {
        loop {
            let result = match self.messenger.receive() {
                Ok(BrowserMessage::Request(raw_request)) => self.handle_request(raw_request),
                Ok(BrowserMessage::Response(raw_response)) => self.handle_response(raw_response),
                Ok(BrowserMessage::Notification(raw_notification)) => self.handle_notification(raw_notification),
                Err(e) => Err(e)
            };

            if let Err(e) = result {
                log::error!("Renderer error encountered: {:?}", e);
                break
            }
        }
    }
}
