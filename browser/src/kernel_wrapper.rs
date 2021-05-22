use super::window::{Window, UIMessage};
use super::kernel::Kernel;
use super::messenger::Messenger;
use message::*;
use std::sync::{Arc, Mutex};

pub struct KernelWrapper {
    window: Window,
    kernel: Arc<Mutex<Kernel>>,
}

impl KernelWrapper {
    pub fn new() -> Self {
        let window = Window::new();
        let kernel = Arc::new(Mutex::new(Kernel::new(window.get_message_sender())));

        Self {
            window,
            kernel,
        }
    }

    pub fn manual_load(&mut self, html: String, css: String) {
        let renderer_id = {
            let mut kernel = self.kernel.lock().unwrap();
            let renderer = kernel.new_renderer();
            renderer.wait_til_ready();

            renderer.id().to_string()
        };

        let kernel = self.kernel.lock().unwrap();

        if let Some(conn) = kernel.get_connection(&renderer_id) {
            Messenger::send_notification::<LoadFile>(conn, &LoadFileContentParams {
                content: html,
                content_type: "text/html".to_string()
            }).unwrap();

            Messenger::send_notification::<LoadFile>(conn, &LoadFileContentParams {
                content: css,
                content_type: "text/css".to_string()
            }).unwrap();

            std::thread::sleep(std::time::Duration::from_nanos(100));
        }
    }

    pub fn run_event_loop(&mut self, viewport: (u32, u32)) {
        let kernel_clone = self.kernel.clone();
        let window_sender = self.window.get_message_sender();

        std::thread::spawn(move || {
            kernel_clone
                .lock()
                .expect("Unable to lock kernel to run loop")
                .run_loop();

            // exit if kernel loop ends before window loop
            window_sender.send(UIMessage::Exit).unwrap();
        });

        self.window.run_loop(viewport);
        self.kernel
            .lock()
            .expect("Unable to lock kernel for clean up")
            .clean_up();
    }
}
