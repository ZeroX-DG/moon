use super::messenger::Callback;

pub struct RendererClient {
    id: String,
    process: std::process::Child,
    ready: bool,
    request_id: u64,
    callbacks: Vec<Callback>
}

impl RendererClient {
    pub fn new() -> Self {
        let current_exe = std::env::current_exe().unwrap();
        let command = std::process::Command::new(current_exe)
            .args(&["render"])
            .spawn()
            .expect("Unable to spawn renderer");
        Self {
            id: nanoid::nanoid!(),
            process: command,
            ready: false,
            request_id: 0,
            callbacks: Vec::new()
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn disconnect(&mut self) {
        self.process.kill().unwrap();
    }

    pub fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
    }

    pub fn request_new_id(&mut self) -> u64 {
        self.request_id += 1;
        self.request_id
    }

    pub fn add_callback(&mut self, callback: Callback) {
        self.callbacks.push(callback);
    }

    pub fn wait_til_ready(&self) {
        loop {
            if self.ready {
                break
            }
            std::thread::sleep(std::time::Duration::from_nanos(10));
        }
    }
}

impl Drop for RendererClient {
    fn drop(&mut self) {
        self.process.kill().unwrap();
    }
}

