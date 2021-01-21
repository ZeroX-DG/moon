// use super::renderer_handler::{RendererHandler, RendererHandlers};
// use flume::Sender;
// use ipc::Selector;
// use message::RendererMessage;

// pub struct Kernel {
//     renderer_handlers: RendererHandlers,
// }

// pub fn select(renderers: &[RendererHandler]) -> (usize, RendererMessage) {
//     let mut selector = Selector::new();
//     for (index, renderer) in renderers.iter().enumerate() {
//         selector = selector.recv(renderer.receiver(), move |msg| {
//             let msg = msg.expect("Error while selecting message");
//             (index, msg)
//         });
//     }
//     selector.wait()
// }

// impl Kernel {
//     fn handle_renderer_msg(
//         &mut self,
//         index: usize,
//         msg: RendererMessage,
//         ui_sender: &Sender<Vec<u8>>,
//     ) {
//         let handler = &self.renderer_handlers[index];
//         match msg {
//             RendererMessage::RePaint(data) => {
//                 ui_sender.send(data).unwrap();
//             }
//             RendererMessage::ResourceNotFound(path) => {
//                 panic!("Resource not found: {:#?}", path);
//             }
//             _ => {}
//         }
//     }
// }

// impl Kernel {
//     pub fn new() -> Self {
//         Self {
//             renderer_handlers: RendererHandlers::new(),
//         }
//     }

//     pub fn renderer_handlers(&mut self) -> &mut RendererHandlers {
//         &mut self.renderer_handlers
//     }

//     pub fn main_loop(&mut self, ui_sender: Sender<Vec<u8>>) {
//         loop {
//             if self.renderer_handlers.is_empty() {
//                 std::thread::sleep(std::time::Duration::from_millis(10));
//                 continue;
//             }
//             let (index, msg) = select(self.renderer_handlers.inner());
//             self.handle_renderer_msg(index, msg, &ui_sender);
//         }
//     }
// }
