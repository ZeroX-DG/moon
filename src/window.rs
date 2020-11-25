use skulpin::winit::{
    self,
    event_loop::{EventLoop, ControlFlow},
    event::{Event, WindowEvent},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use skulpin::{
    Renderer as SkulpinRenderer, RendererBuilder, WinitWindow, PresentMode,
    CoordinateSystem
};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use super::painter::SkiaPainter;
use flume::Receiver;

pub struct WindowWrapper {
    window: winit::window::Window,
    skulpin_renderer: SkulpinRenderer,
    painter: Rc<RefCell<SkiaPainter>>
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
            skulpin_renderer,
            painter: Rc::new(RefCell::new(SkiaPainter::new()))
        }
    }

    pub fn draw_frame(&mut self, display_list: &painting::DisplayList) -> bool {
        let winit_window = WinitWindow::new(&self.window);

        let mut painter = self.painter.borrow_mut();

        let error = self.skulpin_renderer.draw(&winit_window, |canvas, _| {
            painting::paint(display_list, &mut *painter, canvas);
        }).is_err();
        
        if error {
            return false;
        }
        true
    }
}

pub fn run_ui_loop(kernel_receiver: Receiver<painting::DisplayList>) {
    let event_loop = EventLoop::<()>::with_user_event();
    let mut window_wrapper = WindowWrapper::new(&event_loop);
    let need_redraw = Arc::new(Mutex::new(true));
    let display_list = Arc::new(Mutex::new(vec![]));

    let need_redraw_1 = Arc::clone(&need_redraw);
    let display_list_1 = Arc::clone(&display_list);
    std::thread::spawn(move || {
        match kernel_receiver.recv() {
            Ok(new_display_list) => {
                let mut need_redraw = need_redraw_1.lock().unwrap();
                let mut display_list = display_list_1.lock().unwrap();
                *need_redraw = true;
                *display_list = new_display_list;
            }
            _ => {}
        }
    });

    event_loop.run(move |e, _window_target, control_flow| {
        let need_redraw = Arc::clone(&need_redraw);
        let display_list = Arc::clone(&display_list);
        let frame_start = Instant::now();

        match e {
            Event::LoopDestroyed => std::process::exit(0),
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit
            }
            _ => {}
        }

        let is_need_draw = *need_redraw.lock().unwrap();

        if is_need_draw {
            if !window_wrapper.draw_frame(&*display_list.lock().unwrap()) {
                *control_flow = ControlFlow::Exit;
            }
            *need_redraw.lock().unwrap() = false;
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
