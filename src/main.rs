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

    let (tx, rx) = flume::bounded::<painting::DisplayList>(1);

    std::thread::spawn(move || {
        kernel.init_ui();
        kernel.main_loop(tx);
    });

    run_ui_loop(rx);
}
