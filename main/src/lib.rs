use gtk::{Application, prelude::*};

mod app;
mod ui;
mod state;

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

        app_runtime.update_state(|state| {
            let initial_url = state.browser.home_url().clone();
            state.new_tab(initial_url, true);
        });
    });

    app.run();
}

