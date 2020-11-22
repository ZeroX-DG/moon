use skulpin::winit::{
    self,
    event_loop::{EventLoop, ControlFlow},
    event::{Event, WindowEvent},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use skulpin::{
    skia_safe::Color,
    Renderer as SkulpinRenderer, RendererBuilder, WinitWindow, PresentMode,
    CoordinateSystem
};
use std::time::{Instant, Duration};

pub struct WindowWrapper {
    window: winit::window::Window,
    skulpin_renderer: SkulpinRenderer,
}

impl WindowWrapper {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let logical_size = LogicalSize::new(500.0, 300.0);

        let winit_window = WindowBuilder::new()
            .with_title("Moon")
            .with_inner_size(logical_size)
            .build(event_loop)
            .expect("Failed to create window");

        let skulpin_renderer = {
            let winit_window_wrapper = WinitWindow::new(&winit_window);
            RendererBuilder::new()
                .prefer_integrated_gpu()
                .use_vulkan_debug_layer(false)
                .present_mode_priority(vec![PresentMode::Immediate])
                .coordinate_system(CoordinateSystem::Logical)
                .build(&winit_window_wrapper)
                .expect("Failed to create renderer")
        };

        Self {
            window: winit_window,
            skulpin_renderer
        }
    }

    pub fn draw_frame(&mut self) -> bool {
        let winit_window = WinitWindow::new(&self.window);

        let error = self.skulpin_renderer.draw(&winit_window, |canvas, _| {
            canvas.clear(Color::WHITE);
        }).is_err();
        
        if error {
            return false;
        }
        true
    }
}

pub fn run_ui_loop() {
    let event_loop = EventLoop::<()>::with_user_event();
    let mut window_wrapper = WindowWrapper::new(&event_loop);

    event_loop.run(move |e, _window_target, control_flow| {
        let frame_start = Instant::now();

        println!("here: {:?}", frame_start);

        match e {
            Event::LoopDestroyed => std::process::exit(0),
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            _ => {}
        }

        if !window_wrapper.draw_frame() {
            *control_flow = ControlFlow::Exit;
        }

        if *control_flow != ControlFlow::Exit {
            let frame_elapsed = frame_start.elapsed();
            let refresh_rate = 60.0;
            let frame_length = Duration::from_secs_f32(1.0 / refresh_rate);

            if frame_elapsed < frame_length {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + frame_length);
            }
        }
    });
}
