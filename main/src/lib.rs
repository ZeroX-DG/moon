use gtk::{prelude::*, Application};

mod app;
mod delayed_task;
mod render_client;
mod state;
mod ui;

pub fn start_main() {
    let app = Application::builder()
        .application_id("org.moon.MoonBrowser")
        .build();

    app.connect_activate(|app| {
        let ui = ui::UI::new(app.clone());
        let app_runtime = app::AppRuntime::init(ui);

        app_runtime.update_state(|state| {
            state.ui.window.show_all();
            state.ui.window.present();
            state.ui.set_loading_finished();

            if let Some(screen) = state.ui.window.screen() {
                let current_monitor = std::env::var("ACTIVE_MONITOR")
                    .map(|monitor_id| monitor_id.parse().unwrap_or(0))
                    .unwrap_or(0);
                let monitor_geometry = screen.monitor_geometry(current_monitor);
                let (window_x, window_y) = state.ui.window.position();
                let x = monitor_geometry.x() + window_x;
                let y = monitor_geometry.y() + window_y;
                state.ui.window.move_(x, y);
                state.ui.window.set_keep_above(true);
                state.ui.window.show_all();
                state.ui.window.set_keep_above(false);
            }
        });
    });

    app.run();
}
