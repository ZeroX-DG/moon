mod logging;
mod window;
mod renderer_handler;
mod kernel;
mod painter;

use logging::init_logging;
use kernel::Kernel;
use window::run_ui_loop;

fn main() {
    init_logging();

    let mut kernel = Kernel::new();
    kernel.renderer_handlers().new_renderer();

    let (tx, rx) = flume::bounded::<painting::DisplayList>(10);

    std::thread::spawn(move || {
        kernel.main_loop(tx);
        kernel.clean_up();
    });

    run_ui_loop(rx);
}
