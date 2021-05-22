use glutin::event_loop::ControlFlow;
use glutin::event::StartCause;
use glium::Surface;
use std::time::{Instant, Duration};
use flume::{Receiver, Sender};
use super::display_frame::{DisplayFrame, Pixel};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

const DEFAULT_TITLE: &str = "Moon";
const DEFAULT_SIZE: (u32, u32) = (500, 300);

pub type Image = Vec<Pixel>;

pub enum UIMessage {
    Frame(Image),
    Exit
}

pub struct Window {
    message_channel: (Sender<UIMessage>, Receiver<UIMessage>),
    display_frame: Arc<Mutex<DisplayFrame>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            message_channel: flume::bounded(1),
            display_frame: Arc::new(Mutex::new(DisplayFrame::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1))),
        }
    }

    pub fn get_message_sender(&self) -> Sender<UIMessage> {
        self.message_channel.0.clone()
    }

    pub fn run_loop(&mut self, size: (u32, u32)) {
        let (width, height) = size;
        let (_, message_receiver) = self.message_channel.clone();

        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title(DEFAULT_TITLE)
            .with_inner_size(glutin::dpi::LogicalSize::new(
                width as f64,
                height as f64,
            ))
            .with_resizable(false);
        let context_builder = glutin::ContextBuilder::new()
            .with_vsync(true);
        let display = glium::Display::new(window_builder, context_builder, &event_loop).unwrap();

        let mut next_frame_time = Instant::now();

        let texture = glium::Texture2d::empty_with_format(
            &display,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height,
        )
        .unwrap();

        let display_frame_write = self.display_frame.clone();
        let display_frame_read = self.display_frame.clone();

        let should_exit = Arc::new(AtomicBool::new(false));
        let should_exit_clone = should_exit.clone();

        std::thread::spawn(move || {
            match message_receiver.recv() {
                Ok(UIMessage::Frame(image)) => {
                    display_frame_write.lock().unwrap().set_data(image);
                }
                Ok(UIMessage::Exit) => {
                    should_exit.store(true, Ordering::Relaxed);
                }
                Err(e) => panic!("{:?}", e)
            }
        });

        event_loop.run(move |event, _, control_flow| match event {
            glutin::event::Event::NewEvents(StartCause::ResumeTimeReached { .. })
            | glutin::event::Event::NewEvents(StartCause::Init) => {
                next_frame_time += Duration::from_nanos(16_666_667);
                *control_flow = ControlFlow::WaitUntil(next_frame_time);

                if should_exit_clone.load(Ordering::Relaxed) {
                    *control_flow = ControlFlow::Exit;
                    return
                }

                texture.write(
                    glium::Rect {
                        left: 0,
                        bottom: 0,
                        width,
                        height,
                    },
                    &*display_frame_read.lock().unwrap(),
                );

                let target = display.draw();
                texture
                    .as_surface()
                    .fill(&target, glium::uniforms::MagnifySamplerFilter::Linear);
                target.finish().unwrap();
            }
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        });
    }
}

