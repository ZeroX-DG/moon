use std::{time::Duration, sync::{Arc, Mutex}};

pub struct DelayedTask {
    should_run: Arc<Mutex<bool>>
}

impl DelayedTask {
    pub fn new<F: FnOnce() + 'static + Send + Copy>(timeout: Duration, callback: F) -> Self {
        let should_run = Arc::new(Mutex::new(true));
        let should_run_clone = should_run.clone();
        let _ = std::thread::spawn(move || {
            std::thread::sleep(timeout);
            if !*should_run_clone.lock().unwrap() {
                return;
            }
            callback();
        });
        Self {
            should_run,
        }
    }

    pub fn clear(&self) {
        *self.should_run.lock().unwrap() = false;
    }
}