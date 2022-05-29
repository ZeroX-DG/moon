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
        });
    });

    app.run();
}
