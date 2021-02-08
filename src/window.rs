use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{SurfaceTexture, Pixels};
use flume::Receiver;
use std::sync::{Arc, Mutex};

pub fn run_ui_loop() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Arc::new(Mutex::new(Pixels::new(500, 300, surface_texture).unwrap()))
    };

    let pixels_clone = pixels.clone();

    // std::thread::spawn(move || loop {
    //     match pixels_receiver.recv() {
    //         Ok(frame) => {
    //             pixels_clone.lock().unwrap().get_frame().copy_from_slice(&frame);
    //         }
    //         _ => {}
    //     }
    // });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
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
