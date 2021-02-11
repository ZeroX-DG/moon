use flume::{Receiver, Sender};
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{KernelAction, UIAction};

pub fn run_ui_loop(tx_kernel: Sender<KernelAction>, rx_ui: Receiver<UIAction>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Moon")
        .with_inner_size(LogicalSize::new(500.0, 300.0))
        .build(&event_loop)
        .unwrap();
    let window = Arc::new(window);

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &*window);
        Arc::new(Mutex::new(
            Pixels::new(window_size.width, window_size.height, surface_texture).unwrap(),
        ))
    };

    let pixels_clone = pixels.clone();
    let window_clone = window.clone();

    std::thread::spawn(move || loop {
        match rx_ui.recv() {
            Ok(UIAction::RePaint(data)) => {
                pixels_clone
                    .lock()
                    .unwrap()
                    .get_frame()
                    .copy_from_slice(&data);
                window_clone.request_redraw();
            }
            _ => {}
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                // Tell the kernel to clean up when the user close the window
                tx_kernel.send(KernelAction::CleanUp).unwrap();

                *control_flow = ControlFlow::Exit;
            }
            Event::RedrawRequested(_) => {
                if pixels.lock().unwrap().render().is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => (),
        }
    });
}
