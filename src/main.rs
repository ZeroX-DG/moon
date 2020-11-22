mod logging;
mod window;
mod renderer;

use logging::init_logging;
use renderer::Renderers;
use message::KernelMessage;
// use window::run_ui_loop;

pub struct Kernel {
    renderers: Renderers
}

fn main() {
    init_logging();

    let mut kernel = Kernel {
        renderers: Renderers::new()
    };

    let ui_renderer = kernel.renderers.new_renderer();

    ui_renderer.send(KernelMessage::LoadUrl("https://google.com".to_string()))
        .expect("Can't send");

    println!("{:?}", ui_renderer.recv());

    ui_renderer.send(KernelMessage::Exit)
        .expect("Can't send");

    // run_ui_loop();
    clean_up(&mut kernel);
}

fn clean_up(kernel: &mut Kernel) {
    kernel.renderers.close_all();
}
