use ipc::IpcKernel;
use message::MoonMessage;
use std::process::Command;
use skulpin::winit;
use std::thread;

fn main() {
    let ipc = IpcKernel::<MoonMessage>::new();

    // Command::new("target/debug/rendering")
    //     .arg(ipc.address())
    //     .spawn()
    //     .expect("Error when starting renderer");
    //
    println!("{}", ipc.address());

    match ipc.client.receiver.recv() {
        Ok(data) => println!("{:#?}", data),
        _ => {}
    }

    //let logical_size = winit::dpi::LogicalSize::new(500.0, 300.0);

    //// window creation
    //let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    //// Create a single window
    //let winit_window = winit::window::WindowBuilder::new()
    //    .with_title("Moon")
    //    .with_inner_size(logical_size)
    //    .build(&event_loop)
    //    .expect("Failed to create window");

    //let window = skulpin::WinitWindow::new(&winit_window);

    //let renderer = skulpin::RendererBuilder::new()
    //    .use_vulkan_debug_layer(false)
    //    .coordinate_system(skulpin::CoordinateSystem::Logical)
    //    .build(&window);

    //// Check if there were error setting up vulkan
    //if let Err(e) = renderer {
    //    println!("Error during renderer construction: {:?}", e);
    //    return;
    //}

    //let mut renderer = renderer.unwrap();

    //thread::spawn(|| {
    //    ipc.
    //});

    //event_loop.run(move |event, _, control_flow| {
    //    let window = skulpin::WinitWindow::new(&winit_window);

    //    match event {
    //        winit::event::Event::WindowEvent {
    //            event: winit::event::WindowEvent::CloseRequested,
    //            ..
    //        } => *control_flow = winit::event_loop::ControlFlow::Exit,
    //        //
    //        // Request a redraw any time we finish processing events
    //        //
    //        winit::event::Event::MainEventsCleared => {
    //            // Queue a RedrawRequested event.
    //            winit_window.request_redraw();
    //        }
    //        //
    //        // Redraw
    //        //
    //        winit::event::Event::RedrawRequested(_window_id) => {
    //            if let Err(e) = renderer.draw(&window, |canvas, _| {
    //            }) {
    //                println!("Error during draw: {:?}", e);
    //                *control_flow = winit::event_loop::ControlFlow::Exit
    //            }
    //        }
    //        _ => {}
    //    }
    //});
}
