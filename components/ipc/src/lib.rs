/// Inter-process communication
///
/// This module aim to provide an IPC for the kernel
/// and the rendering engine. Since we are using a
/// multi-process architecture like chrome, we must
/// provide some way to communicate between the host
/// process & multiple rendering process.

// Example:
// 
// ```rs
// use ipc::{IpcKernel, IpcRenderer, Message};
// 
// fn main() {
//   if (is_kernel) {
//     let ipc = IpcKernel::new();
//     let child = spawn_renderer(ipc.address());
//
//     ipc.send(data);
//
//     match ipc.recv() {
//       Some(m) => println!("{:#?}", m),
//       None => {}
//     }
//   } else {
//     let ipc = IpcRenderer::new(args.address);
//
//     match ipc.recv() {
//       Some(m) => ipc.send(m),
//       None => {}
//     }
//   }
// }
// ```
