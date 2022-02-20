use gtk::glib;

use crate::{state::AppState, ui::UI};

static mut APP_RUNTIME: Option<AppRuntime> = None;

#[derive(Clone)]
pub struct AppRuntime(glib::Sender<Box<dyn FnOnce(&mut AppState)>>);

impl AppRuntime {
    pub fn init(ui: UI) -> Self {
        let (app_tx, app_rx) = glib::MainContext::channel(Default::default());
        let app_runtime = Self(app_tx);
        let mut state = AppState::new(ui, app_runtime.clone());

        unsafe {
            APP_RUNTIME = Some(app_runtime.clone());
        }

        app_rx.attach(None, move |update_state| {
            update_state(&mut state);

            glib::Continue(true)
        });

        app_runtime
    }

    pub fn update_state(&self, action: impl FnOnce(&mut AppState) + 'static) {
        self.0.send(Box::new(action)).unwrap();
    }
}

pub fn get_app_runtime() -> &'static AppRuntime {
    unsafe {
        APP_RUNTIME
            .as_ref()
            .expect("AppRuntime has not been initialized")
    }
}

